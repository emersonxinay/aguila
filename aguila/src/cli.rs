use crate::analyzer::Analizador;
use crate::interpreter::Interprete;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Obtener ruta de caché para un archivo .ag
fn get_cache_path(archivo: &str) -> Result<PathBuf, String> {
    // Usar .aguila/cache/ en el directorio del proyecto
    let ag_path = Path::new(archivo);
    let mut cache_dir = ag_path.parent().unwrap_or(Path::new(".")).to_path_buf();
    cache_dir.push(".aguila");
    cache_dir.push("cache");

    // Crear directorios si no existen
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Error creando directorio de caché: {}", e))?;

    // Nombre del binario: nombre_archivo sin extensión
    let stem = ag_path.file_stem().unwrap_or_default().to_string_lossy();
    cache_dir.push(stem.to_string());

    Ok(cache_dir)
}

/// Verificar si el binario en caché está actualizado
fn is_cache_valid(ag_file: &str, cache_path: &Path) -> bool {
    // Si no existe el binario, no es válido
    if !cache_path.exists() {
        return false;
    }

    // Comparar timestamps
    let ag_metadata = match fs::metadata(ag_file) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let cache_metadata = match fs::metadata(cache_path) {
        Ok(m) => m,
        Err(_) => return false,
    };

    let ag_modified = ag_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let cache_modified = cache_metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

    // Caché es válido si es más reciente que el archivo .ag
    cache_modified >= ag_modified
}

pub fn cli_ejecutar(archivo: &str) -> Result<(), String> {
    // SISTEMA HÍBRIDO:
    // 1. Intenta usar caché compilado (nativo, ultra-rápido)
    // 2. Si no existe o es viejo, compila automáticamente
    // 3. Fallback a intérprete si la compilación falla

    let cache_path = get_cache_path(archivo)?;

    // Revisar si existe binario compilado y está actualizado
    if is_cache_valid(archivo, &cache_path) {
        // ✅ Ejecutar binario compilado (ultrarrápido)
        let status = std::process::Command::new(&cache_path)
            .status()
            .map_err(|e| format!("Error ejecutando binario compilado: {}", e))?;

        if !status.success() {
            return Err(format!(
                "Binario compilado falló con estado: {:?}",
                status.code()
            ));
        }
        return Ok(());
    }

    // ❌ No hay caché válido, intentar compilar
    let contenido = fs::read_to_string(archivo)
        .map_err(|e| format!("Error al leer archivo '{}': {}", archivo, e))?;

    let mut lexer = Lexer::nuevo(&contenido);
    // Ejecutar directamente con el intérprete/VM
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
                Err(format!(
                    "Se encontraron {} errores de análisis",
                    errores.len()
                ))
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

    // Usar compilador AOT nativo en lugar de JavaScript
    // let exe_path = archivo.replace(".ag", "");
    // let mut compilador = crate::compiler_c::CompiladorAOT::new();
    // compilador.compilar(&programa, &exe_path)?;

    Err("Compilación AOT deshabilitada temporalmente.".to_string())

    // Ok(())
}

pub fn ejecutar_codigo(codigo: &str) -> Result<(), String> {
    let mut lexer = Lexer::nuevo(codigo);
    let tokens = lexer.tokenizar();

    let mut parser = Parser::nuevo(tokens);
    let programa = parser.parsear()?;

    // VM Pipeline
    let compiler = crate::compiler_bytecode::Compiler::new();
    let chunk = compiler.compile(programa);
    let chunk_arc = std::sync::Arc::new(chunk);

    let mut vm = crate::vm::vm::VM::new();

    // Registrar módulos standard
    crate::stdlib::net::registrar_modulo(&mut vm);
    crate::stdlib::thread::registrar_modulo(&mut vm);
    crate::stdlib::mate::registrar_modulo(&mut vm);

    // Reloj
    let reloj_id = vm.registrar_nativa(|_vm, _args| {
        let start = std::time::SystemTime::now();
        let since_the_epoch = start.duration_since(std::time::UNIX_EPOCH).unwrap();
        Ok(crate::vm::value::Value::numero(
            since_the_epoch.as_secs_f64(),
        ))
    });
    vm.globals.insert(
        "reloj".to_string(),
        crate::vm::value::Value::nativa(reloj_id as u32),
    );

    match vm.interpretar(chunk_arc) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error de ejecución: {}", e)),
    }
}
pub fn cli_dev(archivo: &str) {
    println!("Iniciando modo desarrollo para '{}'...", archivo);
    // Placeholder para dev server
    if let Err(e) = cli_compilar(archivo) {
        eprintln!("Error inicial: {}", e);
    }
}

pub fn cli_clean(opcion: Option<&str>) -> Result<(), String> {
    // limpiar            → Limpia recursivamente todos los .aguila/cache/ encontrados
    // limpiar <archivo>  → Limpia solo el caché de ese archivo

    match opcion {
        Some(archivo) => {
            // Limpiar solo un archivo específico
            let cache_path = get_cache_path(archivo)?;
            if cache_path.exists() {
                fs::remove_file(&cache_path)
                    .map_err(|e| format!("Error eliminando caché de '{}': {}", archivo, e))?;
                println!("✓ Caché de '{}' eliminado", archivo);
            } else {
                println!("No hay caché para '{}'", archivo);
            }
            Ok(())
        }
        None => {
            // Buscar y limpiar todos los directorios .aguila/cache/ recursivamente
            let mut cleaned = 0;

            // Buscar en el directorio actual
            for entry in walkdir::WalkDir::new(".")
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if path.ends_with(".aguila/cache") {
                    match fs::remove_dir_all(path) {
                        Ok(_) => {
                            cleaned += 1;
                            println!("✓ Limpiado: {}", path.display());
                        }
                        Err(e) => {
                            eprintln!("⚠ Error limpiando {}: {}", path.display(), e);
                        }
                    }
                }
            }

            if cleaned == 0 {
                println!("No hay caché para limpiar");
            } else {
                println!("✓ Total: {} directorios de caché limpiados", cleaned);
            }
            Ok(())
        }
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
            let chunk_arc = std::sync::Arc::new(chunk);

            let mut vm = crate::vm::vm::VM::new();

            // Registrar módulos estándar
            crate::stdlib::net::registrar_modulo(&mut vm);
            crate::stdlib::thread::registrar_modulo(&mut vm);

            // Registrar función nativa reloj
            let reloj_id = vm.registrar_nativa(|_vm, _args| {
                let start = std::time::SystemTime::now();
                let since_the_epoch = start.duration_since(std::time::UNIX_EPOCH).unwrap();
                Ok(crate::vm::value::Value::numero(
                    since_the_epoch.as_secs_f64(),
                ))
            });
            vm.globals.insert(
                "reloj".to_string(),
                crate::vm::value::Value::nativa(reloj_id as u32),
            );

            match vm.interpretar(chunk_arc) {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    Err(format!("Error de ejecución: {}", e))
                }
            }
        }
        Err(e) => Err(format!("Error de sintaxis: {}", e)),
    }
}

pub fn cli_rustc(archivo: &str) -> Result<(), String> {
    let contenido = match fs::read_to_string(archivo) {
        Ok(c) => c,
        Err(e) => return Err(format!("Error al leer archivo '{}': {}", archivo, e)),
    };

    let mut lexer = Lexer::nuevo(&contenido);
    let tokens = lexer.tokenizar();
    let mut parser = Parser::nuevo(tokens);

    match parser.parsear() {
        Ok(programa) => {
            // Compilar a Rust code
            let mut rust_compiler = crate::compiler_rust::CompilerRust::new();
            let rust_code = rust_compiler.compile(programa);

            // Crear archivo temporal con el código Rust
            let temp_file = "/tmp/aguila_temp.rs";
            fs::write(temp_file, &rust_code)
                .map_err(|e| format!("Error escribiendo archivo temporal: {}", e))?;

            // Compilar con rustc
            let output = std::process::Command::new("rustc")
                .arg("-O") // Optimizaciones
                .arg(temp_file)
                .arg("-o")
                .arg("/tmp/aguila_temp")
                .output()
                .map_err(|e| format!("Error compilando con rustc: {}", e))?;

            if !output.status.success() {
                let err = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Error de compilación rustc:\n{}", err));
            }

            // Ejecutar binario compilado
            let exec_output = std::process::Command::new("/tmp/aguila_temp")
                .output()
                .map_err(|e| format!("Error ejecutando binario: {}", e))?;

            print!("{}", String::from_utf8_lossy(&exec_output.stdout));
            if !exec_output.status.success() {
                eprintln!("{}", String::from_utf8_lossy(&exec_output.stderr));
            }

            Ok(())
        }
        Err(e) => Err(format!("Error de sintaxis: {}", e)),
    }
}
