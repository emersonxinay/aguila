pub mod lista;
pub mod mate;
pub mod texto;
pub mod net;
pub mod thread;

use crate::vm::value::{NativeFn, Obj, Value};
use std::collections::HashMap;

pub fn resolver_metodo_nativo(
    objeto: &Value,
    nombre_metodo: &str,
) -> Option<Value> {
    if objeto.es_texto() {
        return texto::resolver(nombre_metodo);
    } else if let Obj::Lista(_) = objeto.a_objeto() {
        return lista::resolver(nombre_metodo);
    } else if let Obj::Diccionario(_) = objeto.a_objeto() {
        // TODO: Diccionarios
        return None;
    }
    None
}
