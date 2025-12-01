use crate::vm::value::Value;
use crate::vm::vm::{Module, VM};
use std::collections::HashMap;
use std::thread;

pub fn registrar_modulo(vm: &mut VM) {
    let mut exports = HashMap::new();

    let idx_crear = vm.registrar_nativa(thread_crear);
    exports.insert("crear".to_string(), Value::nativa(idx_crear as u32));

    vm.registrar_modulo(Module {
        name: "hilo".to_string(),
        exports,
    });
}

fn thread_crear(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    // args[0] es el módulo 'hilo' (self)
    // args[1] es la función a ejecutar
    // args[2..] son los argumentos para la función

    if args.len() < 2 {
        return Err("hilo.crear requiere al menos 1 argumento (funcion)".to_string());
    }

    let func_val = args[1];
    if !func_val.es_numero() {
        return Err(format!(
            "Primer argumento debe ser una funcion (numero), recibido: {:?}",
            func_val
        ));
    }
    let func_addr = func_val.a_numero() as usize;

    // Argumentos para la función
    let mut thread_args = args[2..].to_vec();

    // Identificar y mover recursos
    let mut moved_resources: Vec<(usize, Box<dyn std::any::Any + Send>)> = Vec::new();

    for (i, arg) in thread_args.iter_mut().enumerate() {
        if arg.es_recurso() {
            let handle = arg.a_recurso_handle();
            // Extraer recurso de la VM actual (Move Semantics)
            if let Some(res_opt) = vm.resources.get_mut(handle as usize) {
                if let Some(res) = res_opt.take() {
                    moved_resources.push((i, res));
                } else {
                    return Err(format!("Recurso {} ya fue movido o es inválido", handle));
                }
            } else {
                return Err(format!("Handle de recurso inválido: {}", handle));
            }
        }
    }

    // Clonar el Chunk (código)
    let chunk_arc = if let Some(chunk) = &vm.chunk {
        chunk.clone()
    } else {
        return Err("No se puede crear hilo: VM no tiene código cargado".to_string());
    };

    // Spawn thread
    thread::spawn(move || {
        let mut thread_vm = VM::new();
        crate::stdlib::net::registrar_modulo(&mut thread_vm);
        crate::stdlib::thread::registrar_modulo(&mut thread_vm);

        thread_vm.chunk = Some(chunk_arc);

        // Re-insertar recursos movidos y actualizar argumentos
        for (arg_idx, res) in moved_resources {
            let new_handle = thread_vm.alloc_resource(res);
            thread_args[arg_idx] = Value::recurso(new_handle);
        }

        // Cargar argumentos en registros
        for (i, arg) in thread_args.iter().enumerate() {
            thread_vm.registers[i] = *arg;
        }

        match thread_vm.run_from(func_addr) {
            Ok(_) => {}
            Err(e) => eprintln!("Error en hilo: {}", e),
        }
    });

    Ok(Value::nulo())
}
