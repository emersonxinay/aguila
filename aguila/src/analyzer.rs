use crate::ast::{Expresion, Programa, Sentencia};
use std::collections::{HashMap, HashSet};

pub struct Analizador {
    errores: Vec<String>,
    scopes: Vec<HashSet<String>>,
    funciones: HashMap<String, usize>, // Nombre -> Aridad
}

impl Analizador {
    pub fn nuevo() -> Self {
        Analizador {
            errores: Vec::new(),
            scopes: vec![HashSet::new()], // Scope global
            funciones: HashMap::new(),
        }
    }

    pub fn analizar(&mut self, programa: &Programa) -> Vec<String> {
        for sentencia in &programa.sentencias {
            self.analizar_sentencia(sentencia);
        }
        self.errores.clone()
    }

    fn entrar_scope(&mut self) {
        self.scopes.push(HashSet::new());
    }

    fn salir_scope(&mut self) {
        self.scopes.pop();
    }

    fn declarar_variable(&mut self, nombre: String) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(nombre);
        }
    }

    fn variable_definida(&self, nombre: &str) -> bool {
        for scope in self.scopes.iter().rev() {
            if scope.contains(nombre) {
                return true;
            }
        }
        false
    }

    fn analizar_sentencia(&mut self, sentencia: &Sentencia) {
        match sentencia {
            Sentencia::Asignacion {
                nombre,
                valor,
                tipo,
            } => {
                self.analizar_expresion(valor);
                self.declarar_variable(nombre.clone());

                if let Some(tipo_declarado) = tipo {
                    match valor {
                        Expresion::Numero(_) if tipo_declarado != "Numero" => {
                            self.errores.push(format!("Error: Tipo incorrecto para variable '{}'. Se esperaba '{}', se encontró 'Numero'", nombre, tipo_declarado));
                        }
                        Expresion::Texto(_) if tipo_declarado != "Texto" => {
                            self.errores.push(format!("Error: Tipo incorrecto para variable '{}'. Se esperaba '{}', se encontró 'Texto'", nombre, tipo_declarado));
                        }
                        Expresion::Logico(_) if tipo_declarado != "Logico" => {
                            self.errores.push(format!("Error: Tipo incorrecto para variable '{}'. Se esperaba '{}', se encontró 'Logico'", nombre, tipo_declarado));
                        }
                        _ => {} // Otros casos o expresiones complejas (por ahora ignorados)
                    }
                }
            }
            Sentencia::Funcion {
                nombre,
                parametros,
                bloque,
                ..
            } => {
                self.declarar_variable(nombre.clone());
                self.funciones.insert(nombre.clone(), parametros.len());

                self.entrar_scope();
                for (param, _) in parametros {
                    self.declarar_variable(param.clone());
                }
                for sent in bloque {
                    self.analizar_sentencia(sent);
                }
                self.salir_scope();
            }
            Sentencia::Si {
                condicion,
                si_bloque,
                sino_bloque,
            } => {
                self.analizar_expresion(condicion);
                self.entrar_scope();
                for sent in si_bloque {
                    self.analizar_sentencia(sent);
                }
                self.salir_scope();

                if let Some(bloque) = sino_bloque {
                    self.entrar_scope();
                    for sent in bloque {
                        self.analizar_sentencia(sent);
                    }
                    self.salir_scope();
                }
            }
            Sentencia::Mientras { condicion, bloque } => {
                self.analizar_expresion(condicion);
                self.entrar_scope();
                for sent in bloque {
                    self.analizar_sentencia(sent);
                }
                self.salir_scope();
            }
            Sentencia::Imprimir(expr) => {
                self.analizar_expresion(expr);
            }
            Sentencia::Expresion(expr) => {
                self.analizar_expresion(expr);
            }
            Sentencia::Retorno(expr_opt) => {
                if let Some(expr) = expr_opt {
                    self.analizar_expresion(expr);
                }
            }
            _ => {}
        }
    }

    fn analizar_expresion(&mut self, expr: &Expresion) {
        match expr {
            Expresion::Identificador(nombre) => {
                if !self.variable_definida(nombre) {
                    self.errores
                        .push(format!("Error: Variable '{}' no definida", nombre));
                }
            }
            Expresion::Llamada { nombre, args } => {
                if !self.variable_definida(nombre) {
                    self.errores
                        .push(format!("Error: Función '{}' no definida", nombre));
                } else if let Some(&aridad) = self.funciones.get(nombre) {
                    if args.len() != aridad {
                        self.errores.push(format!(
                            "Error: Función '{}' espera {} argumentos, pero se recibieron {}",
                            nombre,
                            aridad,
                            args.len()
                        ));
                    }
                }
                for arg in args {
                    self.analizar_expresion(arg);
                }
            }
            Expresion::BinOp { izq, der, .. } => {
                self.analizar_expresion(izq);
                self.analizar_expresion(der);
            }
            _ => {}
        }
    }
}
