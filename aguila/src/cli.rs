use std::fs;
use std::io::{self, Write};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interprete;
use crate::compiler;
use crate::analyzer::Analizador;
use std::path::Path;
use notify::{Watcher, RecursiveMode, Result as NotifyResult};
use std::sync::mpsc::channel;

pub fn cli_ejecutar(archivo: &str) -> Result<(), String> {
    let contenido = fs::read_to_string(archivo)
        .map_err(|e| format!("Error al leer archivo '{}': {}", archivo, e))?;

    ejecutar_codigo(&contenido)
}

pub fn cli_chequear(archivo: &str) -> Result<(), String> {
    let contenido = match fs::read_to_string(archivo) {
        Ok(c) => c,
        Err(e) => return Err(format!("Error al leer archivo '{}': {}", archivo, e)),
    };

    let mut lexer = Lexer::nuevo(&contenido);
    let tokens = lexer.tokenizar();
    let mut parser = Parser::nuevo(tokens);

    match parser.parsear() {
        Ok(programa) => {
            let mut analizador = Analizador::nuevo();
            let errores = analizador.analizar(&programa);

            if errores.is_empty() {
                println!("✅ Análisis estático completado sin errores.");
                Ok(())
            } else {
                println!("❌ Se encontraron {} errores:", errores.len());
                for error in &errores {
                    println!("  - {}", error);
                }
                Err(format!("Se encontraron {} errores de análisis", errores.len()))
            }
        }
        Err(e) => Err(format!("Error de parseo: {}", e)),
    }
}

pub fn cli_compilar(archivo: &str) -> Result<(), String> {
    let contenido = fs::read_to_string(archivo)
        .map_err(|e| format!("Error al leer archivo '{}': {}", archivo, e))?;

    let mut lexer = Lexer::nuevo(&contenido);
    let tokens = lexer.tokenizar();

    let mut parser = Parser::nuevo(tokens);
    let programa = parser.parsear()?;

    let js_codigo = compiler::compilar(programa);

    let archivo_salida = archivo.replace(".ag", ".js");
    fs::write(&archivo_salida, js_codigo)
        .map_err(|e| format!("Error al escribir archivo '{}': {}", archivo_salida, e))?;

    println!("Compilado exitosamente a: {}", archivo_salida);
    Ok(())
}

pub fn ejecutar_codigo(codigo: &str) -> Result<(), String> {
    let mut lexer = Lexer::nuevo(codigo);
    let tokens = lexer.tokenizar();

    let mut parser = Parser::nuevo(tokens);
    let programa = parser.parsear()?;

    let mut interprete = Interprete::nuevo();
    let _ = interprete.ejecutar(programa)?;

    Ok(())
}
pub fn cli_dev(archivo: &str) {
    println!("Iniciando modo desarrollo para '{}'...", archivo);
    // Placeholder para dev server
    if let Err(e) = cli_compilar(archivo) {
        eprintln!("Error inicial: {}", e);
    }
}

pub fn cli_vm(archivo: &str) -> Result<(), String> {
    let contenido = match fs::read_to_string(archivo) {
        Ok(c) => c,
        Err(e) => return Err(format!("Error al leer archivo '{}': {}", archivo, e)),
    };

    let mut lexer = Lexer::nuevo(&contenido);
    let tokens = lexer.tokenizar();
    let mut parser = Parser::nuevo(tokens);

    match parser.parsear() {
        Ok(programa) => {
            let compiler = crate::compiler_bytecode::Compiler::new();
            let chunk = compiler.compile(programa);
            
            let mut vm = crate::vm::vm::VM::new();
            vm.run(&chunk)?;
            Ok(())
        }
        Err(e) => Err(format!("Error de sintaxis: {}", e)),
    }
}
