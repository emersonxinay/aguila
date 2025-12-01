use crate::vm::value::Value;
use crate::vm::vm::VM;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

// Funciones del módulo 'net'

use std::collections::HashMap;

// --- Funciones del Módulo 'net' ---

pub fn net_escuchar(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    // args: [modulo, puerto]
    if args.len() < 2 {
        return Err("net.escuchar requiere 1 argumento (puerto)".to_string());
    }
    let puerto = args[1].a_numero() as u16;
    let addr = format!("0.0.0.0:{}", puerto);

    match TcpListener::bind(&addr) {
        Ok(listener) => {
            listener.set_nonblocking(false).map_err(|e| e.to_string())?;
            let handle = vm.alloc_resource(Box::new(listener));
            Ok(Value::recurso(handle))
        }
        Err(e) => Err(format!("Error al escuchar en {}: {}", addr, e)),
    }
}

pub fn net_conectar(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    // args: [modulo, host, puerto]
    if args.len() < 3 {
        return Err("net.conectar requiere 2 argumentos (host, puerto)".to_string());
    }
    // args[0] es el módulo
    let host_val = args[1];
    let puerto_val = args[2];

    if !host_val.es_objeto() {
        return Err("Host debe ser texto".to_string());
    }
    let host = host_val.a_texto();
    let puerto = puerto_val.a_numero() as u16;
    let addr = format!("{}:{}", host, puerto);

    match TcpStream::connect(&addr) {
        Ok(stream) => {
            stream.set_nonblocking(false).map_err(|e| e.to_string())?;
            let handle = vm.alloc_resource(Box::new(stream));
            Ok(Value::recurso(handle))
        }
        Err(e) => Err(format!("Error al conectar a {}: {}", addr, e)),
    }
}

// --- Métodos de Instancia (Socket/Listener) ---

pub fn socket_aceptar(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    // args[0] es 'self' (el recurso)
    if args.is_empty() {
        return Err("Se requiere 'self'".to_string());
    }
    let handle = args[0].a_recurso_handle();

    // Intentar obtener TcpListener
    // Necesitamos sacar el listener, aceptar, y volver a ponerlo?
    // O usar get_resource_mut?
    // TcpListener::accept(&self)

    let res = if let Some(listener) = vm.get_resource::<TcpListener>(handle) {
        match listener.accept() {
            Ok((stream, addr)) => {
                // println!("Conexión aceptada de {}", addr);
                stream.set_nonblocking(false).map_err(|e| e.to_string())?;
                let stream_handle = vm.alloc_resource(Box::new(stream));
                Ok(Value::recurso(stream_handle))
            }
            Err(e) => Err(format!("Error en aceptar: {}", e)),
        }
    } else {
        Err("El recurso no es un Servidor (TcpListener)".to_string())
    };
    res
}

pub fn socket_escribir(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("escribir requiere (self, datos)".to_string());
    }
    let handle = args[0].a_recurso_handle();

    // Datos: Asumimos string.
    // Necesitamos Value::a_texto() o similar.
    // Hack MVP: Si es numero, escribir numero como string.
    // Si es string (puntero), necesitamos dereferenciar.
    // Vamos a implementar un helper en VM o Value para leer strings.

    // Por ahora, escribimos "HOLA" hardcoded para probar conectividad si no podemos leer el string.
    // O mejor, implementamos Value::as_str unsafe.

    let data = if args[1].es_objeto() {
        args[1].a_texto().as_bytes()
    } else {
        // Fallback or error
        return Err("Datos deben ser texto".to_string());
    };

    let res = if let Some(stream) = vm.get_resource_mut::<TcpStream>(handle) {
        match stream.write(data) {
            Ok(n) => Ok(Value::numero(n as f64)),
            Err(e) => Err(format!("Error al escribir: {}", e)),
        }
    } else {
        Err("El recurso no es un Socket (TcpStream)".to_string())
    };
    res
}

pub fn socket_leer(vm: &mut VM, args: &[Value]) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("leer requiere (self)".to_string());
    }
    let handle = args[0].a_recurso_handle();

    let res = if let Some(stream) = vm.get_resource_mut::<TcpStream>(handle) {
        let mut buffer = [0; 1024];
        match stream.read(&mut buffer) {
            Ok(n) => {
                let s = String::from_utf8_lossy(&buffer[0..n]).to_string();
                Ok(Value::texto(s))
            }
            Err(e) => Err(format!("Error al leer: {}", e)),
        }
    } else {
        Err("El recurso no es un Socket (TcpStream)".to_string())
    };
    res
}

pub fn registrar_modulo(vm: &mut VM) {
    let mut exports = HashMap::new();

    let idx_escuchar = vm.registrar_nativa(net_escuchar);
    exports.insert("escuchar".to_string(), Value::nativa(idx_escuchar as u32));

    let idx_conectar = vm.registrar_nativa(net_conectar);
    exports.insert("conectar".to_string(), Value::nativa(idx_conectar as u32));

    vm.registrar_modulo(crate::vm::vm::Module {
        name: "net".to_string(),
        exports,
    });

    // Métodos de recursos
    let idx_aceptar = vm.registrar_nativa(socket_aceptar);
    vm.registrar_metodo_recurso("aceptar", idx_aceptar);

    let idx_escribir = vm.registrar_nativa(socket_escribir);
    vm.registrar_metodo_recurso("escribir", idx_escribir);

    let idx_leer = vm.registrar_nativa(socket_leer);
    vm.registrar_metodo_recurso("leer", idx_leer);
}
