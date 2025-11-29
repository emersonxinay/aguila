mod ast;
mod lexer;
mod parser;
mod types;
mod interpreter;
mod cli;
mod compiler;
mod compiler_bytecode;
mod analyzer;
mod repl;
mod vm;

use std::env;
use cli::{cli_ejecutar, cli_chequear, cli_compilar, cli_dev, cli_vm};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Modo REPL por defecto
        repl::iniciar();
        return;
    }

    let comando = &args[1];
    match comando.as_str() {
        "ejecutar" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila ejecutar <archivo.ag>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_ejecutar(archivo) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        "vm" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila vm <archivo.ag>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_vm(archivo) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        "chequear" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila chequear <archivo.ag>");
                return;
            }
            let archivo = &args[2];
            cli_chequear(archivo);
        },
        "compilar" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila compilar <archivo.ag>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_compilar(archivo) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            } else {
                println!("Compilación exitosa.");
            }
        },
        "dev" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila dev <archivo.ag>");
                return;
            }
            let archivo = &args[2];
            cli_dev(archivo);
        },
        "--version" => {
            println!("aguila v{}", VERSION);
        }
        "--help" | "-h" => {
            mostrar_ayuda();
        }
        // Si el primer argumento termina en .ag, ejecutarlo directamente
        arg if arg.ends_with(".ag") => {
             if let Err(e) = cli::cli_ejecutar(arg) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Comando desconocido: {}", args[1]);
            mostrar_ayuda();
        }
    }
}

fn mostrar_ayuda() {
    println!("ÁGUILA v{}", VERSION);
    println!();
    println!("Uso:");
    println!("  aguila                   Inicia el intérprete interactivo (REPL)");
    println!("  aguila <archivo.ag>      Ejecuta un archivo directamente");
    println!();
    println!("Comandos explícitos:");
    println!("  ejecutar <archivo.ag>    Ejecuta un archivo ÁGUILA");
    println!("  compilar <archivo.ag>    Compila un archivo a JavaScript");
    println!("  chequear <archivo.ag>    Analiza estáticamente un archivo");
    println!("  repl                     Inicia el intérprete interactivo");
    println!("  --version                Muestra la versión");
    println!("  --help, -h               Muestra esta ayuda");
}
