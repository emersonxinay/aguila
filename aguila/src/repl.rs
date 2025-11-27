use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::interpreter::Interprete;

pub fn iniciar() {
    println!("🦅 ÁGUILA v{}", env!("CARGO_PKG_VERSION"));
    println!("Escribe 'salir' para terminar, o 'ayuda' para ver comandos.");

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Error al inicializar el editor: {}", e);
            return;
        }
    };

    let mut interprete = Interprete::nuevo();
    let mut buffer_multilinea = String::new();

    loop {
        let prompt = if buffer_multilinea.is_empty() {
            ">> "
        } else {
            ".. "
        };

        let readline = rl.readline(prompt);
        match readline {
            Ok(linea) => {
                let linea_trimmed = linea.trim();
                
                // Comandos especiales solo funcionan cuando no hay buffer
                if buffer_multilinea.is_empty() {
                    match linea_trimmed {
                        "" => continue,
                        "salir" => {
                            println!("¡Hasta luego! 🦅");
                            break;
                        }
                        "ayuda" => {
                            mostrar_ayuda();
                            continue;
                        }
                        "limpiar" => {
                            print!("\x1B[2J\x1B[1;1H");
                            continue;
                        }
                        _ => {}
                    }
                }

                // Agregar al historial
                if let Err(e) = rl.add_history_entry(&linea) {
                    eprintln!("Error al agregar al historial: {}", e);
                }

                // Agregar línea al buffer
                if !buffer_multilinea.is_empty() {
                    buffer_multilinea.push('\n');
                }
                buffer_multilinea.push_str(&linea);

                // Verificar si el bloque está completo
                if es_bloque_completo(&buffer_multilinea) {
                    ejecutar_linea(&mut interprete, &buffer_multilinea);
                    buffer_multilinea.clear();
                }
            }
            Err(ReadlineError::Interrupted) => {
                if !buffer_multilinea.is_empty() {
                    println!("Bloque cancelado.");
                    buffer_multilinea.clear();
                } else {
                    println!("CTRL-C detectado. Usa 'salir' para terminar.");
                }
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D detectado. ¡Hasta luego! 🦅");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn es_bloque_completo(codigo: &str) -> bool {
    let mut nivel_llaves = 0;
    let mut nivel_parentesis = 0;
    let mut nivel_corchetes = 0;
    let mut en_string = false;
    let mut escape = false;

    for c in codigo.chars() {
        if escape {
            escape = false;
            continue;
        }

        match c {
            '\\' if en_string => escape = true,
            '"' => en_string = !en_string,
            '{' if !en_string => nivel_llaves += 1,
            '}' if !en_string => nivel_llaves -= 1,
            '(' if !en_string => nivel_parentesis += 1,
            ')' if !en_string => nivel_parentesis -= 1,
            '[' if !en_string => nivel_corchetes += 1,
            ']' if !en_string => nivel_corchetes -= 1,
            _ => {}
        }
    }

    // El bloque está completo si todos los delimitadores están balanceados
    nivel_llaves == 0 && nivel_parentesis == 0 && nivel_corchetes == 0 && !en_string
}

fn mostrar_ayuda() {
    println!("╔═══════════════════════════════════════════╗");
    println!("║         Comandos del REPL de Águila       ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  salir     - Termina la sesión            ║");
    println!("║  ayuda     - Muestra este mensaje         ║");
    println!("║  limpiar   - Limpia la pantalla           ║");
    println!("║  ↑/↓       - Navega por el historial      ║");
    println!("║  ←/→       - Mueve el cursor               ║");
    println!("║  CTRL-C    - Cancela bloque multilínea    ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  Tip: Escribe bloques de código en        ║");
    println!("║  múltiples líneas. El prompt cambia a     ║");
    println!("║  '..' mientras espera el cierre.          ║");
    println!("╚═══════════════════════════════════════════╝");
}

fn ejecutar_linea(interprete: &mut Interprete, codigo: &str) {
    let mut lexer = Lexer::nuevo(codigo);
    let tokens = lexer.tokenizar();

    let mut parser = Parser::nuevo(tokens);
    match parser.parsear() {
        Ok(programa) => {
            match interprete.ejecutar(programa) {
                Ok(val_opt) => {
                    if let Some(val) = val_opt {
                        println!("=> {}", val.a_texto());
                    }
                }
                Err(e) => eprintln!("❌ Error: {}", e),
            }
        }
        Err(e) => eprintln!("❌ Sintaxis: {}", e),
    }
}
