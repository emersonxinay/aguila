mod analyzer;
mod ast;
mod cli;
mod compiler;
mod compiler_bytecode;
mod interpreter;
mod lexer;
mod parser;
mod repl;
mod types;
mod vm;

use cli::{cli_chequear, cli_compilar, cli_dev, cli_ejecutar, cli_vm};
use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Modo REPL por defecto
        repl::iniciar().await;
        return;
    }

    let comando = &args[1];
    match comando.as_str() {
        "ejecutar" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila ejecutar <archivo.agl>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_ejecutar(archivo).await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "vm" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila vm <archivo.agl>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_vm(archivo) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "chequear" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila chequear <archivo.agl>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_chequear(archivo) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "compilar" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila compilar <archivo.agl>");
                return;
            }
            let archivo = &args[2];
            if let Err(e) = cli_compilar(archivo) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            } else {
                println!("Compilación exitosa.");
            }
        }
        "dev" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila dev <archivo.agl>");
                return;
            }
            let archivo = &args[2];
            cli_dev(archivo);
        }
        "--version" => {
            println!("aguila v{}", VERSION);
        }
        "--help" | "-h" => {
            mostrar_ayuda();
        }
        // Si el primer argumento termina en .agl, ejecutarlo directamente
        arg if arg.ends_with(".agl") => {
            if let Err(e) = cli::cli_ejecutar(arg).await {
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
    println!("  aguila <archivo.agl>      Ejecuta un archivo directamente");
    println!();
    println!("Comandos explícitos:");
    println!("  ejecutar <archivo.agl>    Ejecuta un archivo ÁGUILA");
    println!("  compilar <archivo.agl>    Compila un archivo a JavaScript");
    println!("  chequear <archivo.agl>    Analiza estáticamente un archivo");
    println!("  repl                     Inicia el intérprete interactivo");
    println!("  --version                Muestra la versión");
    println!("  --help, -h               Muestra esta ayuda");
}
