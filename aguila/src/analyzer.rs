use crate::ast::{Expresion, Literal, Programa, Sentencia};
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
                objetivo, valor, ..
            } => {
                self.analizar_expresion(valor);
                if let Expresion::Identificador(nombre) = objetivo {
                    self.declarar_variable(nombre.clone());
                }
            }
            Sentencia::Funcion {
                nombre,
                params,
                cuerpo,
                ..
            } => {
                self.declarar_variable(nombre.clone());
                self.funciones.insert(nombre.clone(), params.len());

                self.entrar_scope();
                for param in params {
                    self.declarar_variable(param.nombre.clone());
                }
                for sent in cuerpo {
                    self.analizar_sentencia(sent);
                }
                self.salir_scope();
            }
            Sentencia::Si {
                condicion,
                entonces,
                sino,
            } => {
                self.analizar_expresion(condicion);
                self.entrar_scope();
                for sent in entonces {
                    self.analizar_sentencia(sent);
                }
                self.salir_scope();

                if let Some(bloque) = sino {
                    self.entrar_scope();
                    for sent in bloque {
                        self.analizar_sentencia(sent);
                    }
                    self.salir_scope();
                }
            }
            Sentencia::Mientras { condicion, cuerpo } => {
                self.analizar_expresion(condicion);
                self.entrar_scope();
                for sent in cuerpo {
                    self.analizar_sentencia(sent);
                }
                self.salir_scope();
            }
            Sentencia::Imprimir(exprs) => {
                for expr in exprs {
                    self.analizar_expresion(expr);
                }
            }
            Sentencia::Expresion(expr) => {
                self.analizar_expresion(expr);
            }
            Sentencia::Retornar(expr_opt) => {
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
            Expresion::Llamada { func, args } => {
                if let Expresion::Identificador(nombre) = &**func {
                    if !self.variable_definida(nombre) {
                        // Podría ser nativa, así que no marcamos error estricto si no está en funciones map
                        // Pero si está en funciones map, verificamos aridad
                        if let Some(&aridad) = self.funciones.get(nombre) {
                            if args.len() != aridad {
                                self.errores.push(format!(
                                    "Error: Función '{}' espera {} argumentos, pero se recibieron {}",
                                    nombre, aridad, args.len()
                                ));
                            }
                        }
                    }
                }
                self.analizar_expresion(func);
                for arg in args {
                    self.analizar_expresion(arg);
                }
            }
            Expresion::Binaria { izq, der, .. } => {
                self.analizar_expresion(izq);
                self.analizar_expresion(der);
            }
            Expresion::Literal(_) => {}
            _ => {}
        }
    }
}
