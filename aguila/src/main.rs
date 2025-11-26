mod ast;
mod lexer;
mod parser;
mod types;
mod interpreter;
mod cli;

use std::env;

const VERSION: &str = "0.1.0";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // Si no hay argumentos, iniciar REPL por defecto
        cli::cli_repl();
        return;
    }

    match args[1].as_str() {
        "ejecutar" => {
            if args.len() < 3 {
                eprintln!("Uso: aguila ejecutar <archivo.ag>");
                return;
            }
            if let Err(e) = cli::cli_ejecutar(&args[2]) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        "repl" => {
            cli::cli_repl();
        }
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
    println!("  repl                     Inicia el intérprete interactivo");
    println!("  --version                Muestra la versión");
    println!("  --help, -h               Muestra esta ayuda");
}
