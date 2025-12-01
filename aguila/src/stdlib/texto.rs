use crate::vm::value::{NativeFn, Obj, Value};

pub fn resolver(nombre: &str) -> Option<Value> {
    match nombre {
        "mayusculas" => Some(Value::objeto(Obj::MetodoNativo(mayusculas))),
        "minusculas" => Some(Value::objeto(Obj::MetodoNativo(minusculas))),
        "longitud" => Some(Value::objeto(Obj::MetodoNativo(longitud))),
        "contiene" => Some(Value::objeto(Obj::MetodoNativo(contiene))),
        "reemplazar" => Some(Value::objeto(Obj::MetodoNativo(reemplazar))),
        "recortar" => Some(Value::objeto(Obj::MetodoNativo(recortar))),
        "dividir" => Some(Value::objeto(Obj::MetodoNativo(dividir))),
        _ => None,
    }
}

fn mayusculas(args: Vec<Value>) -> Result<Value, String> {
    // args[0] es 'self' (el string)
    if args.len() < 1 {
        return Err("Se requiere el objeto texto".to_string());
    }
    let s = args[0].a_texto();
    Ok(Value::texto(s.to_uppercase()))
}

fn minusculas(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el objeto texto".to_string());
    }
    let s = args[0].a_texto();
    Ok(Value::texto(s.to_lowercase()))
}

fn longitud(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el objeto texto".to_string());
    }
    let s = args[0].a_texto();
    Ok(Value::entero(s.chars().count() as i32))
}

fn contiene(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("Se requiere argumento 'subtexto'".to_string());
    }
    let s = args[0].a_texto();
    let sub = if args[1].es_texto() {
        args[1].a_texto()
    } else {
        return Err("El argumento debe ser texto".to_string());
    };
    Ok(Value::logico(s.contains(sub)))
}

fn reemplazar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("Se requieren argumentos 'viejo' y 'nuevo'".to_string());
    }
    let s = args[0].a_texto();
    let viejo = args[1].a_texto();
    let nuevo = args[2].a_texto();
    Ok(Value::texto(s.replace(viejo, nuevo)))
}

fn recortar(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 1 {
        return Err("Se requiere el objeto texto".to_string());
    }
    let s = args[0].a_texto();
    Ok(Value::texto(s.trim().to_string()))
}

fn dividir(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("Se requiere argumento 'separador'".to_string());
    }
    let s = args[0].a_texto();
    let sep = args[1].a_texto();
    
    let partes: Vec<Value> = s
        .split(sep)
        .map(|p| Value::texto(p.to_string()))
        .collect();
        
    Ok(Value::objeto(Obj::Lista(partes)))
}
