use crate::vm::value::{NativeFn, Obj, Value};
use crate::vm::vm::VM;
use rand::Rng;
use std::collections::HashMap;

pub fn resolver(nombre: &str) -> Option<Value> {
    match nombre {
        "raiz" => Some(Value::objeto(Obj::MetodoNativo(raiz))),
        "aleatorio" => Some(Value::objeto(Obj::MetodoNativo(aleatorio))),
        "sen" => Some(Value::objeto(Obj::MetodoNativo(sen))),
        "cos" => Some(Value::objeto(Obj::MetodoNativo(cos))),
        "piso" => Some(Value::objeto(Obj::MetodoNativo(piso))),
        "techo" => Some(Value::objeto(Obj::MetodoNativo(techo))),
        "potencia" => Some(Value::objeto(Obj::MetodoNativo(potencia))),
        _ => None,
    }
}

pub fn registrar_modulo(vm: &mut VM) {
    let mut exports = HashMap::new();

    exports.insert("raiz".to_string(), Value::objeto(Obj::MetodoNativo(raiz)));
    exports.insert("aleatorio".to_string(), Value::objeto(Obj::MetodoNativo(aleatorio)));
    exports.insert("sen".to_string(), Value::objeto(Obj::MetodoNativo(sen)));
    exports.insert("cos".to_string(), Value::objeto(Obj::MetodoNativo(cos)));
    exports.insert("piso".to_string(), Value::objeto(Obj::MetodoNativo(piso)));
    exports.insert("techo".to_string(), Value::objeto(Obj::MetodoNativo(techo)));
    exports.insert("potencia".to_string(), Value::objeto(Obj::MetodoNativo(potencia)));

    vm.registrar_modulo(crate::vm::vm::Module {
        name: "mate".to_string(),
        exports,
    });
}

fn raiz(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el número".to_string());
    }
    let n = args[0].a_numero();
    Ok(Value::numero(n.sqrt()))
}

fn aleatorio(_args: Vec<Value>) -> Result<Value, String> {
    let mut rng = rand::thread_rng();
    Ok(Value::numero(rng.gen()))
}

fn sen(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el ángulo".to_string());
    }
    let n = args[0].a_numero();
    Ok(Value::numero(n.sin()))
}

fn cos(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el ángulo".to_string());
    }
    let n = args[0].a_numero();
    Ok(Value::numero(n.cos()))
}

fn piso(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el número".to_string());
    }
    let n = args[0].a_numero();
    Ok(Value::entero(n.floor() as i32))
}

fn techo(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el número".to_string());
    }
    let n = args[0].a_numero();
    Ok(Value::entero(n.ceil() as i32))
}

fn potencia(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("Se requieren base y exponente".to_string());
    }
    let base = args[0].a_numero();
    let exp = args[1].a_numero();
    Ok(Value::numero(base.powf(exp)))
}
