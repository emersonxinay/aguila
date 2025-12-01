use crate::ast::{Expresion, Literal, OperadorBinario, OperadorUnario, Programa, Sentencia};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::{NativeFn, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::rc::Rc;

pub struct Interprete {
    variables: Vec<Rc<RefCell<HashMap<String, Value>>>>,
    funciones: HashMap<String, (Vec<crate::ast::Parametro>, Vec<Sentencia>, bool)>,
    clases: HashMap<String, (Vec<String>, Vec<Sentencia>)>,
    retorno_actual: Option<Value>,
    romper_actual: bool,
    continuar_actual: bool,
}

impl Interprete {
    pub fn nuevo() -> Self {
        let mut interprete = Interprete {
            variables: vec![Rc::new(RefCell::new(HashMap::new()))],
            funciones: HashMap::new(),
            clases: HashMap::new(),
            retorno_actual: None,
            romper_actual: false,
            continuar_actual: false,
        };
        interprete.registrar_funciones_nativas();
        interprete
    }

    fn registrar_funciones_nativas(&mut self) {
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert(
                "imprimir".to_string(),
                Value::FuncionNativa(NativeFn(Rc::new(|args| {
                    for arg in args {
                        print!("{} ", arg.a_texto());
                    }
                    println!();
                    Ok(Value::Nulo)
                }))),
            );
        }
    }

    pub fn ejecutar(&mut self, programa: Programa) -> Result<Option<Value>, String> {
        for sent in programa.sentencias {
            if let Some(val) = self.ejecutar_sentencia(&sent)? {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    fn ejecutar_sentencia(&mut self, sentencia: &Sentencia) -> Result<Option<Value>, String> {
        match sentencia {
            Sentencia::Asignacion {
                objetivo, valor, ..
            } => {
                let val = self.evaluar_expresion(valor)?;
                match objetivo {
                    Expresion::Identificador(nombre) => {
                        if let Some(scope) = self.variables.last_mut() {
                            scope.borrow_mut().insert(nombre.clone(), val);
                        }
                    }
                    _ => return Err("Asignación compleja no soportada aún".to_string()),
                }
                Ok(None)
            }
            Sentencia::Expresion(expr) => {
                self.evaluar_expresion(expr)?;
                Ok(None)
            }
            Sentencia::Imprimir(exprs) => {
                for expr in exprs {
                    let val = self.evaluar_expresion(expr)?;
                    print!("{} ", val.a_texto());
                }
                println!();
                Ok(None)
            }
            Sentencia::Si {
                condicion,
                entonces,
                sino,
            } => {
                let cond = self.evaluar_expresion(condicion)?;
                if cond.a_booleano() {
                    for sent in entonces {
                        if let Some(val) = self.ejecutar_sentencia(sent)? {
                            return Ok(Some(val));
                        }
                    }
                } else if let Some(sino_bloque) = sino {
                    for sent in sino_bloque {
                        if let Some(val) = self.ejecutar_sentencia(sent)? {
                            return Ok(Some(val));
                        }
                    }
                }
                Ok(None)
            }
            Sentencia::Mientras { condicion, cuerpo } => {
                loop {
                    let cond = self.evaluar_expresion(condicion)?;
                    if !cond.a_booleano() {
                        break;
                    }
                    for sent in cuerpo {
                        if let Some(val) = self.ejecutar_sentencia(sent)? {
                            return Ok(Some(val));
                        }
                        if self.romper_actual {
                            self.romper_actual = false;
                            return Ok(None);
                        }
                        if self.continuar_actual {
                            self.continuar_actual = false;
                            break;
                        }
                    }
                    if self.romper_actual {
                        self.romper_actual = false;
                        break;
                    }
                }
                Ok(None)
            }
            Sentencia::Funcion {
                nombre,
                params,
                cuerpo,
                es_async,
                ..
            } => {
                self.funciones
                    .insert(nombre.clone(), (params.clone(), cuerpo.clone(), *es_async));
                Ok(None)
            }
            Sentencia::Retornar(expr_opt) => {
                let val = if let Some(expr) = expr_opt {
                    self.evaluar_expresion(expr)?
                } else {
                    Value::Nulo
                };
                Ok(Some(val))
            }
            _ => Ok(None), // Ignorar otras sentencias por ahora
        }
    }

    pub fn evaluar_expresion(&mut self, expr: &Expresion) -> Result<Value, String> {
        match expr {
            Expresion::Literal(lit) => match lit {
                Literal::Entero(n) => Ok(Value::Numero(*n as f64)),
                Literal::Decimal(n) => Ok(Value::Numero(*n)),
                Literal::Texto(s) => Ok(Value::Texto(s.clone())),
                Literal::Booleano(b) => Ok(Value::Logico(*b)),
                Literal::Nulo => Ok(Value::Nulo),
            },
            Expresion::Identificador(nombre) => {
                for scope in self.variables.iter().rev() {
                    if let Some(val) = scope.borrow().get(nombre) {
                        return Ok(val.clone());
                    }
                }
                Err(format!("Variable no definida: {}", nombre))
            }
            Expresion::Binaria { izq, op, der } => {
                let val_izq = self.evaluar_expresion(izq)?;
                let val_der = self.evaluar_expresion(der)?;
                self.evaluar_binaria(val_izq, op, val_der)
            }
            Expresion::Llamada { func, args } => {
                let nombre = match &**func {
                    Expresion::Identificador(s) => s,
                    _ => return Err("Solo llamadas a identificadores soportadas".to_string()),
                };

                let mut vals_args = Vec::new();
                for arg in args {
                    vals_args.push(self.evaluar_expresion(arg)?);
                }

                if let Some((params, cuerpo, _)) = self.funciones.get(nombre).cloned() {
                    let mut nuevo_scope = HashMap::new();
                    for (param, val) in params.iter().zip(vals_args.iter()) {
                        nuevo_scope.insert(param.nombre.clone(), val.clone());
                    }

                    self.variables.push(Rc::new(RefCell::new(nuevo_scope)));
                    let mut ret = Value::Nulo;
                    for sent in &cuerpo {
                        if let Some(val) = self.ejecutar_sentencia(sent)? {
                            ret = val;
                            break;
                        }
                    }
                    self.variables.pop();
                    Ok(ret)
                } else {
                    // Buscar nativa
                    for scope in self.variables.iter().rev() {
                        if let Some(Value::FuncionNativa(f)) = scope.borrow().get(nombre) {
                            return (f.0)(&vals_args);
                        }
                    }
                    Err(format!("Función no encontrada: {}", nombre))
                }
            }
            _ => Err("Expresión no soportada".to_string()),
        }
    }

    fn evaluar_binaria(
        &self,
        izq: Value,
        op: &OperadorBinario,
        der: Value,
    ) -> Result<Value, String> {
        match op {
            OperadorBinario::Suma => match (izq, der) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Numero(a + b)),
                (Value::Texto(a), Value::Texto(b)) => Ok(Value::Texto(a + &b)),
                _ => Err("Tipos incompatibles para suma".to_string()),
            },
            OperadorBinario::Resta => match (izq, der) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Numero(a - b)),
                _ => Err("Tipos incompatibles para resta".to_string()),
            },
            OperadorBinario::Multiplicacion => match (izq, der) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Numero(a * b)),
                _ => Err("Tipos incompatibles para multiplicación".to_string()),
            },
            OperadorBinario::Division => match (izq, der) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Numero(a / b)),
                _ => Err("Tipos incompatibles para división".to_string()),
            },
            OperadorBinario::Menor => match (izq, der) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Logico(a < b)),
                _ => Err("Tipos incompatibles para menor que".to_string()),
            },
            OperadorBinario::Mayor => match (izq, der) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Logico(a > b)),
                _ => Err("Tipos incompatibles para mayor que".to_string()),
            },
            OperadorBinario::Igual => Ok(Value::Logico(izq == der)),
            _ => Err("Operador no soportado en intérprete básico".to_string()),
        }
    }
}
