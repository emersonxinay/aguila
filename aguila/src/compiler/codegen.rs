use crate::ast::{Expresion, Programa, Sentencia};
use std::collections::HashSet;

#[derive(PartialEq)]
enum TipoScope {
    Global,
    Funcion,
    Bloque,
}

pub struct GeneradorJS {
    codigo: String,
    scopes: Vec<(HashSet<String>, TipoScope)>,
    exportaciones: Vec<String>,
}

impl GeneradorJS {
    pub fn nuevo() -> Self {
        let preambulo = "
const fecha = {
    ahora: () => Date.now(),
    formato: (ts, fmt) => new Date(ts).toISOString() // Simplificado para MVP
};
const imprimir = console.log;
";
        GeneradorJS {
            codigo: String::from("\"use strict\";\n") + preambulo,
            scopes: vec![(HashSet::new(), TipoScope::Global)],
            exportaciones: Vec::new(),
        }
    }

    pub fn generar(&mut self, programa: Programa) -> String {
        for sentencia in programa.sentencias {
            self.generar_sentencia(sentencia);
        }

        // Generar exportaciones al final
        if !self.exportaciones.is_empty() {
            self.codigo.push_str("\nmodule.exports = { ");
            for (i, export) in self.exportaciones.iter().enumerate() {
                if i > 0 {
                    self.codigo.push_str(", ");
                }
                self.codigo.push_str(export);
            }
            self.codigo.push_str(" };\n");
        }

        self.codigo.clone()
    }

    fn entrar_scope(&mut self, tipo: TipoScope) {
        self.scopes.push((HashSet::new(), tipo));
    }

    fn salir_scope(&mut self) {
        self.scopes.pop();
    }

    fn declarar_variable(&mut self, nombre: String) {
        // Si estamos en el scope global (nivel 1), agregamos a exportaciones
        if self.scopes.len() == 1 {
            self.exportaciones.push(nombre.clone());
        }

        if let Some((scope, _)) = self.scopes.last_mut() {
            scope.insert(nombre);
        }
    }

    fn debe_crear_variable(&self, nombre: &str) -> bool {
        // Buscar la variable en los scopes hacia atrás
        for (scope, tipo) in self.scopes.iter().rev() {
            if scope.contains(nombre) {
                // Si la encontramos, verificamos si cruzamos una barrera de función
                // Si estamos en el mismo contexto de función (o global), la reutilizamos.
                // Si la variable viene de una función superior o global, y estamos en una función, la sombreamos (Python-like).
                return false;
            }
            // Si encontramos un límite de función y no hemos encontrado la variable,
            // entonces cualquier variable encontrada más arriba será "externa",
            // por lo que debemos crear una nueva local (shadowing).
            if *tipo == TipoScope::Funcion {
                return true;
            }
        }
        // Si no existe en ningún lado, la creamos
        true
    }

    fn generar_sentencia(&mut self, sentencia: Sentencia) {
        match sentencia {
            Sentencia::Importar { ruta, alias } => {
                let ruta_js = ruta.replace(".agl", ".js");
                if let Some(nombre) = alias {
                    self.codigo
                        .push_str(&format!("const {} = require(\"{}\");\n", nombre, ruta_js));
                    // No exportamos lo importado por defecto, pero sí lo registramos en el scope actual
                    self.declarar_variable(nombre);
                    // Nota: declarar_variable lo agregará a exportaciones si es global.
                    // Tal vez no queremos re-exportar importaciones?
                    // Por simplicidad en MVP, dejemos que se re-exporte si es global.
                } else {
                    self.codigo
                        .push_str(&format!("require(\"{}\");\n", ruta_js));
                }
            }
            Sentencia::Imprimir(expr) => {
                self.codigo.push_str("console.log(");
                self.generar_expresion(expr);
                self.codigo.push_str(");\n");
            }
            Sentencia::Asignacion { nombre, valor, .. } => {
                if self.debe_crear_variable(&nombre) {
                    self.codigo.push_str("let ");
                    self.declarar_variable(nombre.clone());
                }
                self.codigo.push_str(&nombre);
                self.codigo.push_str(" = ");
                self.generar_expresion(valor);
                self.codigo.push_str(";\n");
            }
            Sentencia::Funcion {
                nombre,
                parametros,
                bloque,
                es_asincrona,
                ..
            } => {
                self.declarar_variable(nombre.clone());
                if es_asincrona {
                    self.codigo.push_str("async ");
                }
                self.codigo.push_str(&format!("function {}(", nombre));

                self.entrar_scope(TipoScope::Funcion);

                for (i, (param, _)) in parametros.iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(", ");
                    }
                    self.codigo.push_str(param);
                    self.declarar_variable(param.clone());
                }

                self.codigo.push_str(") {\n");

                for sent in bloque {
                    self.generar_sentencia(sent);
                }

                self.salir_scope();
                self.codigo.push_str("}\n");
            }
            Sentencia::Clase {
                nombre,
                padre,
                metodos,
                ..
            } => {
                self.declarar_variable(nombre.clone());
                self.codigo.push_str(&format!("class {}", nombre));
                if let Some(p) = padre {
                    self.codigo.push_str(&format!(" extends {}", p));
                }
                self.codigo.push_str(" {\n");

                self.entrar_scope(TipoScope::Bloque); // Scope de clase

                for (metodo_nombre, params, cuerpo) in metodos {
                    // Traducir "inicializar" a "constructor"
                    let nombre_js =
                        if metodo_nombre == "inicializar" || metodo_nombre == "constructor" {
                            "constructor".to_string()
                        } else {
                            metodo_nombre
                        };

                    self.codigo.push_str(&format!("    {}(", nombre_js));

                    self.entrar_scope(TipoScope::Funcion);
                    for (i, (param, _)) in params.iter().enumerate() {
                        if i > 0 {
                            self.codigo.push_str(", ");
                        }
                        self.codigo.push_str(param);
                        self.declarar_variable(param.clone());
                    }
                    self.codigo.push_str(") {\n");

                    for sent in cuerpo {
                        self.generar_sentencia(sent);
                    }

                    self.salir_scope(); // Salir de metodo
                    self.codigo.push_str("    }\n");
                }

                self.salir_scope(); // Salir de clase
                self.codigo.push_str("}\n");
            }
            Sentencia::Retorno(expr_opt) => {
                self.codigo.push_str("return");
                if let Some(expr) = expr_opt {
                    self.codigo.push(' ');
                    self.generar_expresion(expr);
                }
                self.codigo.push_str(";\n");
            }
            Sentencia::Expresion(expr) => {
                self.generar_expresion(expr);
                self.codigo.push_str(";\n");
            }
            Sentencia::Si {
                condicion,
                si_bloque,
                sino_bloque,
            } => {
                self.codigo.push_str("if (");
                self.generar_expresion(condicion);
                self.codigo.push_str(") {\n");

                self.entrar_scope(TipoScope::Bloque);
                for sent in si_bloque {
                    self.generar_sentencia(sent);
                }
                self.salir_scope();

                self.codigo.push_str("}");

                if let Some(bloque) = sino_bloque {
                    self.codigo.push_str(" else {\n");
                    self.entrar_scope(TipoScope::Bloque);
                    for sent in bloque {
                        self.generar_sentencia(sent);
                    }
                    self.salir_scope();
                    self.codigo.push_str("}");
                }
                self.codigo.push_str("\n");
            }
            Sentencia::Mientras { condicion, bloque } => {
                self.codigo.push_str("while (");
                self.generar_expresion(condicion);
                self.codigo.push_str(") {\n");

                self.entrar_scope(TipoScope::Bloque);
                for sent in bloque {
                    self.generar_sentencia(sent);
                }
                self.salir_scope();

                self.codigo.push_str("}\n");
            }
            _ => {
                self.codigo.push_str("// Sentencia no soportada aún\n");
            }
        }
    }

    fn generar_expresion(&mut self, expr: Expresion) {
        match expr {
            Expresion::Texto(s) => {
                self.codigo.push('"');
                self.codigo.push_str(&s);
                self.codigo.push('"');
            }
            Expresion::Numero(n) => {
                self.codigo.push_str(&n.to_string());
            }
            Expresion::Logico(b) => {
                self.codigo.push_str(if b { "true" } else { "false" });
            }
            Expresion::Identificador(nombre) => {
                self.codigo.push_str(&nombre);
            }
            Expresion::Lista(elementos) => {
                self.codigo.push('[');
                for (i, elem) in elementos.into_iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(", ");
                    }
                    self.generar_expresion(elem);
                }
                self.codigo.push(']');
            }
            Expresion::Diccionario(pares) => {
                self.codigo.push_str("{\n");
                for (i, (clave, valor)) in pares.into_iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(",\n");
                    }
                    self.codigo.push_str(&format!("\"{}\": ", clave));
                    self.generar_expresion(valor);
                }
                self.codigo.push_str("\n}");
            }
            Expresion::AccesoIndice { objeto, indice } => {
                self.generar_expresion(*objeto);
                self.codigo.push('[');
                self.generar_expresion(*indice);
                self.codigo.push(']');
            }
            Expresion::AccesoAtributo { objeto, atributo } => {
                self.generar_expresion(*objeto);
                self.codigo.push('.');
                self.codigo.push_str(&atributo);
            }
            Expresion::MetodoLlamada {
                objeto,
                metodo,
                args,
            } => {
                self.generar_expresion(*objeto);
                self.codigo.push('.');
                self.codigo.push_str(&metodo);
                self.codigo.push('(');
                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(", ");
                    }
                    self.generar_expresion(arg);
                }
                self.codigo.push(')');
            }
            Expresion::AsignacionAtributo {
                objeto,
                atributo,
                valor,
            } => {
                self.generar_expresion(*objeto);
                self.codigo.push('.');
                self.codigo.push_str(&atributo);
                self.codigo.push_str(" = ");
                self.generar_expresion(*valor);
            }
            Expresion::Instancia { clase, args } => {
                self.codigo.push_str("new ");
                self.codigo.push_str(&clase);
                self.codigo.push('(');
                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(", ");
                    }
                    self.generar_expresion(arg);
                }
                self.codigo.push(')');
            }
            Expresion::BinOp { izq, op, der } => {
                self.codigo.push('(');
                self.generar_expresion(*izq);
                self.codigo.push_str(&format!(" {} ", op));
                self.generar_expresion(*der);
                self.codigo.push(')');
            }
            Expresion::Llamada { nombre, args } => {
                self.codigo.push_str(&nombre);
                self.codigo.push('(');
                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(", ");
                    }
                    self.generar_expresion(arg);
                }
                self.codigo.push(')');
            }
            Expresion::Interpolacion(partes) => {
                self.codigo.push('`');
                for parte in partes {
                    match parte {
                        Expresion::Texto(s) => self.codigo.push_str(&s),
                        _ => {
                            self.codigo.push_str("${");
                            self.generar_expresion(parte);
                            self.codigo.push('}');
                        }
                    }
                }
                self.codigo.push('`');
            }
            Expresion::Esperar(expr) => {
                self.codigo.push_str("(await ");
                self.generar_expresion(*expr);
                self.codigo.push(')');
            }
            Expresion::FuncionAnonima {
                parametros,
                bloque,
                es_asincrona,
            } => {
                if es_asincrona {
                    self.codigo.push_str("async ");
                }
                self.codigo.push_str("function(");
                for (i, param) in parametros.iter().enumerate() {
                    if i > 0 {
                        self.codigo.push_str(", ");
                    }
                    self.codigo.push_str(param);
                }
                self.codigo.push_str(") {\n");

                self.entrar_scope(TipoScope::Funcion);
                // Registrar parámetros en el scope
                for param in &parametros {
                    self.declarar_variable(param.clone());
                }

                for sent in bloque {
                    self.generar_sentencia(sent);
                }

                self.salir_scope();
                self.codigo.push_str("}");
            }
            _ => {
                self.codigo.push_str("null /* Expresión no soportada */");
            }
        }
    }
}
