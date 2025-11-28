use crate::ast::{Token, Sentencia, Expresion, Programa};

pub struct Parser {
    tokens: Vec<Token>,
    posicion: usize,
}

impl Parser {
    pub fn nuevo(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            posicion: 0,
        }
    }

    fn token_actual(&self) -> &Token {
        self.tokens.get(self.posicion).unwrap_or(&Token::EOF)
    }

    fn token_siguiente(&self) -> &Token {
        self.tokens.get(self.posicion + 1).unwrap_or(&Token::EOF)
    }

    fn avanzar(&mut self) -> Token {
        let token = self.token_actual().clone();
        if self.posicion < self.tokens.len() {
            self.posicion += 1;
        }
        token
    }

    fn token_a_string(&self, token: &Token) -> String {
        match token {
            Token::Identificador(s) => format!("identificador '{}'", s),
            Token::Numero(n) => format!("número {}", n),
            Token::Texto(s) => format!("texto \"{}\"", s),
            Token::LlaveAbre => "'{'".to_string(),
            Token::LlaveCierra => "'}'".to_string(),
            Token::ParAbre => "'('".to_string(),
            Token::ParCierra => "')'".to_string(),
            Token::CorcheteAbre => "'['".to_string(),
            Token::CorcheteCierra => "']'".to_string(),
            Token::Coma => "','".to_string(),
            Token::DosPuntos => "':'".to_string(),
            Token::Punto => "'.'".to_string(),
            Token::Flecha => "'->'".to_string(),
            Token::Asignacion => "'='".to_string(),
            Token::Si => "'si'".to_string(),
            Token::Sino => "'sino'".to_string(),
            Token::Mientras => "'mientras'".to_string(),
            Token::Para => "'para'".to_string(),
            Token::Funcion => "'funcion'".to_string(),
            Token::Retornar => "'retornar'".to_string(),
            Token::Clase => "'clase'".to_string(),
            Token::Importar => "'importar'".to_string(),
            Token::EOF => "fin de archivo".to_string(),
            _ => format!("{:?}", token),
        }
    }

    fn esperar(&mut self, esperado: Token) -> Result<(), String> {
        if self.token_actual() == &esperado {
            self.avanzar();
            Ok(())
        } else {
            Err(format!(
                "Error de sintaxis: se esperaba {}, se encontró {}",
                self.token_a_string(&esperado),
                self.token_a_string(self.token_actual())
            ))
        }
    }

    fn esperar_con_contexto(&mut self, esperado: Token, contexto: &str) -> Result<(), String> {
        if self.token_actual() == &esperado {
            self.avanzar();
            Ok(())
        } else {
            Err(format!(
                "Error de sintaxis: se esperaba {} {}. Se encontró {}",
                self.token_a_string(&esperado),
                contexto,
                self.token_a_string(self.token_actual())
            ))
        }
    }

    pub fn parsear(&mut self) -> Result<Programa, String> {
        let mut sentencias = Vec::new();
        while self.token_actual() != &Token::EOF {
            sentencias.push(self.parsear_sentencia()?);
        }
        Ok(Programa { sentencias })
    }

    fn parsear_sentencia(&mut self) -> Result<Sentencia, String> {
        match self.token_actual() {
            Token::Imprimir => self.parsear_imprimir(),
            Token::Si => self.parsear_si(),
            Token::Segun => self.parsear_segun(),
            Token::Mientras => self.parsear_mientras(),
            Token::Para => self.parsear_para(),
            Token::Retornar => self.parsear_retornar(),
            Token::Romper => self.parsear_romper(),
            Token::Continuar => self.parsear_continuar(),
            Token::Funcion => self.parsear_funcion(false),
            Token::Asincrono => {
                self.avanzar(); // Consumir 'asincrono'
                if self.token_actual() == &Token::Funcion {
                    self.parsear_funcion(true)
                } else {
                    Err("Se esperaba 'funcion' después de 'asincrono'".to_string())
                }
            }
            Token::Clase => self.parsear_clase(),
            Token::Importar => self.parsear_importar(),
            Token::Intentar => self.parsear_intentar(),
            Token::Identificador(nombre) => {
                let nombre_clone = nombre.clone();
                
                // Verificar si es asignación simple: nombre = valor
                if self.token_siguiente() == &Token::Asignacion {
                    self.avanzar(); // Consumir nombre
                    self.avanzar(); // Consumir '='
                    let valor = self.parsear_expresion()?;
                    Ok(Sentencia::Asignacion { nombre: nombre_clone, tipo: None, valor })
                } else {
                    // No es asignación simple. Parsear como expresión.
                    // parsear_expresion consumirá el identificador.
                    let expr = self.parsear_expresion()?;
                    
                    // Verificar asignaciones complejas
                    if let Expresion::AccesoIndice { objeto, indice } = &expr {
                        if self.token_actual() == &Token::Asignacion {
                            self.avanzar();
                            let valor = self.parsear_expresion()?;
                            return Ok(Sentencia::AsignacionIndice { objeto: (**objeto).clone(), indice: (**indice).clone(), valor });
                        }
                    }
                    if let Expresion::AccesoAtributo { objeto, atributo } = &expr {
                        if self.token_actual() == &Token::Asignacion {
                            self.avanzar();
                            let valor = self.parsear_expresion()?;
                            return Ok(Sentencia::AsignacionAtributo { objeto: (**objeto).clone(), atributo: atributo.clone(), valor });
                        }
                    }
                    
                    Ok(Sentencia::Expresion(expr))
                }
            },
            _ => {
                // Si no es ninguno de los anteriores, puede ser una expresión (llamada a función, etc.)
                // O una asignación a propiedad/índice que no empieza con identificador simple (aunque raro)
                
                let expr = self.parsear_expresion()?;
                
                // Verificar si es asignación a índice: lista[i] = valor
                if let Expresion::AccesoIndice { objeto, indice } = &expr {
                    if self.token_actual() == &Token::Asignacion {
                        self.avanzar(); // Consumir '='
                        let valor = self.parsear_expresion()?;
                        return Ok(Sentencia::AsignacionIndice {
                            objeto: (**objeto).clone(),
                            indice: (**indice).clone(),
                            valor,
                        });
                    }
                }

                // Verificar si es asignación a propiedad: obj.prop = valor
                if let Expresion::AccesoAtributo { objeto, atributo } = &expr {
                    if self.token_actual() == &Token::Asignacion {
                        self.avanzar(); // Consumir '='
                        let valor = self.parsear_expresion()?;
                        return Ok(Sentencia::AsignacionAtributo {
                            objeto: (**objeto).clone(),
                            atributo: atributo.clone(),
                            valor,
                        });
                    }
                }
                
                Ok(Sentencia::Expresion(expr))
            }
        }
    }

    fn parsear_imprimir(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Imprimir)?;
        let expr = self.parsear_expresion()?;
        Ok(Sentencia::Imprimir(expr))
    }

    fn parsear_importar(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Importar)?;
        let ruta = if let Token::Texto(s) = self.avanzar() {
            s
        } else {
            return Err("Se esperaba ruta de archivo en 'importar'".to_string());
        };

        let alias = if let Token::Identificador(s) = self.token_actual() {
            if s == "como" || s == "as" {
                self.avanzar();
                if let Token::Identificador(nombre) = self.avanzar() {
                    Some(nombre)
                } else {
                    return Err("Se esperaba nombre después de 'como'".to_string());
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(Sentencia::Importar { ruta, alias })
    }

    fn parsear_intentar(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Intentar)?;
        self.esperar(Token::LlaveAbre)?;
        let bloque_intentar = self.parsear_bloque()?;
        self.esperar(Token::LlaveCierra)?;

        self.esperar(Token::Capturar)?;
        let variable_error = if let Token::Identificador(nombre) = self.avanzar() {
            nombre
        } else {
            return Err("Se esperaba nombre de variable de error después de 'capturar'".to_string());
        };

        self.esperar(Token::LlaveAbre)?;
        let bloque_capturar = self.parsear_bloque()?;
        self.esperar(Token::LlaveCierra)?;

        Ok(Sentencia::Intentar {
            bloque_intentar,
            variable_error,
            bloque_capturar,
        })
    }

    fn parsear_retornar(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Retornar)?;
        if self.token_actual() == &Token::LlaveCierra || self.token_actual() == &Token::EOF {
            Ok(Sentencia::Retorno(None))
        } else {
            let expr = self.parsear_expresion()?;
            Ok(Sentencia::Retorno(Some(expr)))
        }
    }

    fn parsear_romper(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Romper)?;
        Ok(Sentencia::Romper)
    }

    fn parsear_continuar(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Continuar)?;
        Ok(Sentencia::Continuar)
    }

    fn parsear_si(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // Consumir 'si'
        let condicion = self.parsear_expresion()?;
        self.esperar_con_contexto(Token::LlaveAbre, "para iniciar el bloque 'si'")?;
        let si_bloque = self.parsear_bloque()?;
        self.esperar(Token::LlaveCierra)?; // Consumir } del bloque si
        
        let mut sino_bloque = None;
        
        // Manejo de 'sino si' y 'sino'
        if self.token_actual() == &Token::Sino {
            self.avanzar(); // Consumir 'sino'
            if self.token_actual() == &Token::Si {
                 // Es un 'sino si', lo tratamos recursivamente como otro 'si' dentro del 'sino'
                 let sentencia_sino_si = self.parsear_si()?;
                 sino_bloque = Some(vec![sentencia_sino_si]);
            } else {
                self.esperar_con_contexto(Token::LlaveAbre, "para iniciar el bloque 'sino'")?;
                sino_bloque = Some(self.parsear_bloque()?);
                self.esperar(Token::LlaveCierra)?; // Consumir } del bloque sino
            }
        }

        Ok(Sentencia::Si {
            condicion,
            si_bloque,
            sino_bloque,
        })
    }

    fn parsear_segun(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Segun)?;
        let expresion = self.parsear_expresion()?;
        self.esperar(Token::LlaveAbre)?;

        let mut casos = Vec::new();
        let mut defecto = None;

        while self.token_actual() != &Token::LlaveCierra {
            // println!("En clase. Token: {:?}", self.token_actual());
            if self.token_actual() == &Token::Caso {
                self.avanzar();
                let valor = self.parsear_expresion()?;
                self.esperar(Token::LlaveAbre)?;
                let bloque = self.parsear_bloque()?;
                self.esperar(Token::LlaveCierra)?;
                casos.push((valor, bloque));
            } else if self.token_actual() == &Token::Defecto {
                self.avanzar();
                self.esperar(Token::LlaveAbre)?;
                let bloque = self.parsear_bloque()?;
                self.esperar(Token::LlaveCierra)?;
                if defecto.is_some() {
                    return Err("Múltiples bloques 'defecto' en 'según'".to_string());
                }
                defecto = Some(bloque);
            } else {
                return Err(format!("Se esperaba 'caso' o 'defecto', se encontró {:?}", self.token_actual()));
            }
        }
        self.esperar(Token::LlaveCierra)?;

        Ok(Sentencia::Segun {
            expresion,
            casos,
            defecto,
        })
    }

    fn parsear_mientras(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // Consumir 'mientras'
        let condicion = self.parsear_expresion()?;
        self.esperar_con_contexto(Token::LlaveAbre, "para iniciar el bloque 'mientras'")?;
        let bloque = self.parsear_bloque()?;
        self.esperar(Token::LlaveCierra)?;
        Ok(Sentencia::Mientras { condicion, bloque })
    }

    fn parsear_para(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Para)?;

        let variable = if let Token::Identificador(nombre) = self.avanzar() {
            nombre
        } else {
            return Err("Se esperaba nombre de variable en 'para'".to_string());
        };

        if self.token_actual() == &Token::Asignacion {
            // para i = 0 hasta 5
            self.avanzar(); // Consumir '='
            
            let inicio = self.parsear_expresion()?;
            
            self.esperar(Token::Hasta)?;
            
            let fin = self.parsear_expresion()?;
            
            self.esperar(Token::LlaveAbre)?;
            let bloque = self.parsear_bloque()?;
            self.esperar(Token::LlaveCierra)?;

            Ok(Sentencia::ParaRango {
                variable,
                inicio,
                fin,
                bloque,
            })
        } else if self.token_actual() == &Token::En {
            // para n en lista
            self.avanzar();
            let iterador = self.parsear_expresion()?;
            self.esperar(Token::LlaveAbre)?;
            let bloque = self.parsear_bloque()?;
            self.esperar(Token::LlaveCierra)?;

            Ok(Sentencia::Para {
                variable,
                iterador,
                bloque,
            })
        } else {
            Err("Se esperaba '=' o 'en' en sentencia 'para'".to_string())
        }
    }

    fn parsear_funcion(&mut self, es_asincrona: bool) -> Result<Sentencia, String> {
        self.avanzar(); // Consumir 'funcion'
        let nombre = if let Token::Identificador(n) = self.avanzar() {
            n
        } else {
            return Err("Se esperaba un nombre para la función".to_string());
        };

        self.esperar_con_contexto(Token::ParAbre, "después del nombre de la función")?;
        let mut parametros = Vec::new();
        if self.token_actual() != &Token::ParCierra {
            loop {
                if let Token::Identificador(param) = self.avanzar() {
                    let mut tipo_param = None;
                    if self.token_actual() == &Token::DosPuntos {
                        self.avanzar();
                        if let Token::Identificador(t) = self.avanzar() {
                            tipo_param = Some(t);
                        } else {
                            return Err("Se esperaba tipo de parámetro".to_string());
                        }
                    }
                    parametros.push((param, tipo_param));
                } else {
                    return Err("Se esperaba nombre de parámetro".to_string());
                }

                if self.token_actual() == &Token::Coma {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        self.esperar_con_contexto(Token::ParCierra, "para cerrar los parámetros")?;

        let mut tipo_retorno = None;
        if self.token_actual() == &Token::Flecha {
            self.avanzar();
            if let Token::Identificador(t) = self.avanzar() {
                tipo_retorno = Some(t);
            } else {
                return Err("Se esperaba tipo de retorno después de '->'".to_string());
            }
        }

        self.esperar_con_contexto(Token::LlaveAbre, "para iniciar el cuerpo de la función")?;
        let bloque = self.parsear_bloque()?;
        self.esperar(Token::LlaveCierra)?; // Consumir } del cuerpo de la función

        Ok(Sentencia::Funcion {
            nombre,
            parametros,
            retorno_tipo: tipo_retorno,
            bloque,
            es_asincrona,
        })
    }

    fn parsear_clase(&mut self) -> Result<Sentencia, String> {
        self.esperar(Token::Clase)?;

        let nombre = if let Token::Identificador(n) = self.avanzar() {
            n
        } else {
            return Err("Se esperaba nombre de clase".to_string());
        };

        let padre = if self.token_actual() == &Token::DosPuntos {
            self.avanzar();
            if let Token::Identificador(p) = self.avanzar() {
                Some(p)
            } else {
                None
            }
        } else {
            None
        };

        self.esperar(Token::LlaveAbre)?;

        let mut atributos = Vec::new();
        let mut metodos = Vec::new();

        while self.token_actual() != &Token::LlaveCierra {
            if self.token_actual() == &Token::Funcion {
                // ... (código existente)
                self.avanzar();
                let metodo_nombre = if let Token::Identificador(n) = self.avanzar() {
                    n
                } else {
                    return Err("Se esperaba nombre de método".to_string());
                };
                
                self.esperar(Token::ParAbre)?;
                let mut params = Vec::new();
                while self.token_actual() != &Token::ParCierra {
                    if let Token::Identificador(param) = self.avanzar() {
                        let tipo = if self.token_actual() == &Token::DosPuntos {
                            self.avanzar();
                            if let Token::Identificador(t) = self.avanzar() { Some(t) } else { None }
                        } else { None };
                        params.push((param, tipo));
                        if self.token_actual() == &Token::Coma { self.avanzar(); }
                    }
                }
                self.esperar(Token::ParCierra)?;
                self.esperar(Token::LlaveAbre)?;
                let bloque = self.parsear_bloque()?;
                self.esperar(Token::LlaveCierra)?;
                metodos.push((metodo_nombre, params, bloque));
            
            } else if self.token_actual() == &Token::Nuevo {
                self.avanzar(); // Consumir 'nuevo'
                self.esperar(Token::ParAbre)?;
                let mut params = Vec::new();
                while self.token_actual() != &Token::ParCierra {
                    if let Token::Identificador(param) = self.avanzar() {
                        let tipo = if self.token_actual() == &Token::DosPuntos {
                            self.avanzar();
                            if let Token::Identificador(t) = self.avanzar() { Some(t) } else { None }
                        } else { None };
                        params.push((param, tipo));
                        if self.token_actual() == &Token::Coma { self.avanzar(); }
                    }
                }
                self.esperar(Token::ParCierra)?;
                self.esperar(Token::LlaveAbre)?;
                let bloque = self.parsear_bloque()?;
                self.esperar(Token::LlaveCierra)?;
                metodos.push(("nuevo".to_string(), params, bloque));

            } else if let Token::Identificador(nombre) = self.token_actual().clone() {
                // Verificar si es método o atributo mirando el siguiente token
                if self.token_siguiente() == &Token::ParAbre {
                    // Es un método: nombre(...) { ... }
                    self.avanzar(); // Consumir nombre
                    self.esperar(Token::ParAbre)?;
                    let mut params = Vec::new();
                    while self.token_actual() != &Token::ParCierra {
                        if let Token::Identificador(param) = self.avanzar() {
                            let tipo = if self.token_actual() == &Token::DosPuntos {
                                self.avanzar();
                                if let Token::Identificador(t) = self.avanzar() { Some(t) } else { None }
                            } else { None };
                            params.push((param, tipo));
                            if self.token_actual() == &Token::Coma { self.avanzar(); }
                        }
                    }
                    self.esperar(Token::ParCierra)?;
                    self.esperar(Token::LlaveAbre)?;
                    let bloque = self.parsear_bloque()?;
                    self.esperar(Token::LlaveCierra)?;
                    metodos.push((nombre, params, bloque));
                } else {
                    // Es un atributo: nombre: Tipo
                    self.avanzar(); // Consumir nombre
                    let tipo = if self.token_actual() == &Token::DosPuntos {
                        self.avanzar();
                        if let Token::Identificador(t) = self.avanzar() { Some(t) } else { None }
                    } else { None };
                    atributos.push((nombre, tipo));
                }
            } else if self.token_actual() == &Token::LlaveAbre {
                // Manejar bloques anidados (ej: inicializadores {})
                self.avanzar();
                let mut balance = 1;
                while balance > 0 && self.token_actual() != &Token::EOF {
                    if self.token_actual() == &Token::LlaveAbre { balance += 1; }
                    else if self.token_actual() == &Token::LlaveCierra { balance -= 1; }
                    
                    if balance > 0 {
                        self.avanzar();
                    }
                }
                if balance == 0 {
                    self.avanzar(); // Consumir última llave cierra
                }
            } else {
                // Token inesperado dentro de clase, avanzar para evitar bucle infinito
                self.avanzar();
            }
        }
        self.esperar(Token::LlaveCierra)?;

        Ok(Sentencia::Clase {
            nombre,
            padre,
            atributos,
            metodos,
        })
    }

    fn parsear_bloque(&mut self) -> Result<Vec<Sentencia>, String> {
        let mut sentencias = Vec::new();
        // println!("Iniciando bloque. Token: {:?}", self.token_actual());
        while self.token_actual() != &Token::LlaveCierra {
            sentencias.push(self.parsear_sentencia()?);
        }
        // println!("Fin bloque. Token: {:?}", self.token_actual());
        Ok(sentencias)
    }

    fn parsear_expresion(&mut self) -> Result<Expresion, String> {
        self.parsear_logica_o()
    }

    fn parsear_logica_o(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_logica_y()?;

        while self.token_actual() == &Token::O {
            self.avanzar();
            let der = self.parsear_logica_y()?;
            izq = Expresion::BinOp {
                izq: Box::new(izq),
                op: "o".to_string(),
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_logica_y(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_comparacion()?;

        while self.token_actual() == &Token::Y {
            self.avanzar();
            let der = self.parsear_comparacion()?;
            izq = Expresion::BinOp {
                izq: Box::new(izq),
                op: "y".to_string(),
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_comparacion(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_suma()?;

        while matches!(
            self.token_actual(),
            Token::Igual | Token::NoIgual | Token::Mayor | Token::Menor | Token::MayorIgual | Token::MenorIgual
        ) {
            let op = match self.avanzar() {
                Token::Igual => "==",
                Token::NoIgual => "!=",
                Token::Mayor => ">",
                Token::Menor => "<",
                Token::MayorIgual => ">=",
                Token::MenorIgual => "<=",
                _ => unreachable!(),
            };
            let der = self.parsear_suma()?;
            izq = Expresion::BinOp {
                izq: Box::new(izq),
                op: op.to_string(),
                der: Box::new(der),
            };
        }

        Ok(izq)
    }

    fn parsear_suma(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_multiplicacion()?;

        while matches!(self.token_actual(), Token::Mas | Token::Menos) {
            let op = match self.avanzar() {
                Token::Mas => "+",
                Token::Menos => "-",
                _ => unreachable!(),
            };
            let der = self.parsear_multiplicacion()?;
            izq = Expresion::BinOp {
                izq: Box::new(izq),
                op: op.to_string(),
                der: Box::new(der),
            };
        }

        Ok(izq)
    }

    fn parsear_multiplicacion(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_potencia()?;

        while matches!(self.token_actual(), Token::Por | Token::Div | Token::Modulo | Token::DivEntera) {
            let op = match self.avanzar() {
                Token::Por => "*",
                Token::Div => "/",
                Token::Modulo => "%",
                Token::DivEntera => "//",
                _ => unreachable!(),
            };
            let der = self.parsear_potencia()?;
            izq = Expresion::BinOp {
                izq: Box::new(izq),
                op: op.to_string(),
                der: Box::new(der),
            };
        }

        Ok(izq)
    }

    fn parsear_potencia(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_unaria()?;

        while self.token_actual() == &Token::Potencia {
            self.avanzar();
            let der = self.parsear_unaria()?;
            izq = Expresion::BinOp {
                izq: Box::new(izq),
                op: "^".to_string(),
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_unaria(&mut self) -> Result<Expresion, String> {
        if self.token_actual() == &Token::Esperar {
            self.avanzar();
            let expr = self.parsear_unaria()?;
            Ok(Expresion::Esperar(Box::new(expr)))
        } else if self.token_actual() == &Token::No {
            self.avanzar();
            let expr = self.parsear_unaria()?;
            Ok(Expresion::UnOp {
                op: "no".to_string(),
                der: Box::new(expr),
            })
        } else if self.token_actual() == &Token::Menos {
            self.avanzar();
            let expr = self.parsear_unaria()?;
            Ok(Expresion::UnOp {
                op: "-".to_string(),
                der: Box::new(expr),
            })
        } else {
            self.parsear_primaria()
        }
    }

    fn parsear_primaria(&mut self) -> Result<Expresion, String> {
        let expr = match self.token_actual().clone() {
            Token::Numero(n) => {
                self.avanzar();
                Expresion::Numero(n)
            }
            Token::Texto(s) => {
                self.avanzar();
                Expresion::Texto(s)
            }
            Token::TextoInterpolado(s) => {
                self.avanzar();
                let mut partes = Vec::new();
                let mut buffer = String::new();
                let mut chars = s.chars().peekable();
                
                while let Some(c) = chars.next() {
                    if c == '{' {
                        if !buffer.is_empty() {
                            partes.push(Expresion::Texto(buffer.clone()));
                            buffer.clear();
                        }
                        
                        // Extraer contenido entre {}
                        let mut expr_str = String::new();
                        let mut balance = 1;
                        while let Some(inner_c) = chars.next() {
                            if inner_c == '{' {
                                balance += 1;
                                expr_str.push(inner_c);
                            } else if inner_c == '}' {
                                balance -= 1;
                                if balance == 0 {
                                    break;
                                }
                                expr_str.push(inner_c);
                            } else {
                                expr_str.push(inner_c);
                            }
                        }
                        
                        // Parsear expresión interna
                        let mut lexer = crate::lexer::Lexer::nuevo(&expr_str);
                        let tokens = lexer.tokenizar();
                        let mut parser = Parser::nuevo(tokens);
                        match parser.parsear_expresion() {
                            Ok(expr) => partes.push(expr),
                            Err(e) => return Err(format!("Error en interpolación: {}", e)),
                        }
                    } else {
                        buffer.push(c);
                    }
                }
                
                if !buffer.is_empty() {
                    partes.push(Expresion::Texto(buffer));
                }
                
                Expresion::Interpolacion(partes)
            }
            Token::Verdadero => {
                self.avanzar();
                Expresion::Logico(true)
            }
            Token::Falso => {
                self.avanzar();
                Expresion::Logico(false)
            }
            Token::Nulo => {
                self.avanzar();
                Expresion::Nulo
            }
            Token::ParAbre => {
                self.avanzar();
                let expr = self.parsear_expresion()?;
                self.esperar(Token::ParCierra)?;
                expr
            }
            Token::CorcheteAbre => {
                self.avanzar();
                let mut elementos = Vec::new();
                while self.token_actual() != &Token::CorcheteCierra {
                    elementos.push(self.parsear_expresion()?);
                    if self.token_actual() == &Token::Coma {
                        self.avanzar();
                    }
                }
                self.esperar(Token::CorcheteCierra)?;
                Expresion::Lista(elementos)
            }
            Token::LlaveAbre => {
                self.avanzar();
                let mut pares = Vec::new();
                while self.token_actual() != &Token::LlaveCierra {
                    let clave = if let Token::Texto(s) = self.avanzar() {
                        s
                    } else if let Token::Identificador(s) = self.token_actual().clone() {
                        self.avanzar();
                        s
                    } else {
                        return Err("Se esperaba clave en diccionario".to_string());
                    };
                    self.esperar(Token::DosPuntos)?;
                    let valor = self.parsear_expresion()?;
                    pares.push((clave, valor));
                    if self.token_actual() == &Token::Coma {
                        self.avanzar();
                    }
                }
                self.esperar(Token::LlaveCierra)?;
                Expresion::Diccionario(pares)
            }
            Token::Identificador(nombre) => {
                self.avanzar();
                if self.token_actual() == &Token::ParAbre {
                    self.avanzar();
                    let mut args = Vec::new();
                    while self.token_actual() != &Token::ParCierra {
                        args.push(self.parsear_expresion()?);
                        if self.token_actual() == &Token::Coma {
                            self.avanzar();
                        }
                    }
                    self.esperar(Token::ParCierra)?;
                    Expresion::Llamada { nombre, args }
                } else {
                    Expresion::Identificador(nombre)
                }
            }
            Token::Nuevo => {
                self.avanzar();
                if let Token::Identificador(clase) = self.avanzar() {
                    self.esperar(Token::ParAbre)?;
                    let mut args = Vec::new();
                    while self.token_actual() != &Token::ParCierra {
                        args.push(self.parsear_expresion()?);
                        if self.token_actual() == &Token::Coma {
                            self.avanzar();
                        }
                    }
                    self.esperar(Token::ParCierra)?;
                    Expresion::Instancia { clase, args }
                } else {
                    return Err("Se esperaba nombre de clase después de 'nuevo'".to_string());
                }
            }
            Token::Funcion => {
                self.avanzar();
                self.esperar(Token::ParAbre)?;
                let mut parametros = Vec::new();
                while self.token_actual() != &Token::ParCierra {
                    if let Token::Identificador(param) = self.avanzar() {
                        parametros.push(param);
                    } else {
                        return Err("Se esperaba nombre de parámetro".to_string());
                    }
                    if self.token_actual() == &Token::Coma {
                        self.avanzar();
                    }
                }
                self.esperar(Token::ParCierra)?;
                self.esperar(Token::LlaveAbre)?;
                let bloque = self.parsear_bloque()?;
                self.esperar(Token::LlaveCierra)?;
                Expresion::FuncionAnonima { parametros, bloque, es_asincrona: false }
            }
            Token::Asincrono => {
                self.avanzar(); // Consumir 'asincrono'
                self.esperar(Token::Funcion)?;
                self.esperar(Token::ParAbre)?;
                let mut parametros = Vec::new();
                while self.token_actual() != &Token::ParCierra {
                    if let Token::Identificador(param) = self.avanzar() {
                        parametros.push(param);
                    } else {
                        return Err("Se esperaba nombre de parámetro".to_string());
                    }
                    if self.token_actual() == &Token::Coma {
                        self.avanzar();
                    }
                }
                self.esperar(Token::ParCierra)?;
                self.esperar(Token::LlaveAbre)?;
                let bloque = self.parsear_bloque()?;
                self.esperar(Token::LlaveCierra)?;
                Expresion::FuncionAnonima { parametros, bloque, es_asincrona: true }
            }
            _ => return Err(format!("Token inesperado en expresión: {:?}", self.token_actual())),
        };

        self.parsear_postfija(expr)
    }

    fn parsear_postfija(&mut self, mut expr: Expresion) -> Result<Expresion, String> {
        loop {
            match self.token_actual() {
                Token::Punto => {
                    self.avanzar();
                    if let Token::Identificador(nombre) = self.avanzar() {
                        if self.token_actual() == &Token::ParAbre {
                            self.avanzar();
                            let mut args = Vec::new();
                            while self.token_actual() != &Token::ParCierra {
                                args.push(self.parsear_expresion()?);
                                if self.token_actual() == &Token::Coma {
                                    self.avanzar();
                                }
                            }
                            self.esperar(Token::ParCierra)?;
                            expr = Expresion::MetodoLlamada {
                                objeto: Box::new(expr),
                                metodo: nombre,
                                args,
                            };
                        } else if self.token_actual() == &Token::Asignacion {
                            self.avanzar();
                            let valor = self.parsear_expresion()?;
                            expr = Expresion::AsignacionAtributo {
                                objeto: Box::new(expr),
                                atributo: nombre,
                                valor: Box::new(valor),
                            };
                            break;
                        } else {
                            expr = Expresion::AccesoAtributo {
                                objeto: Box::new(expr),
                                atributo: nombre,
                            };
                        }
                    }
                }
                Token::CorcheteAbre => {
                    self.avanzar();
                    let indice = self.parsear_expresion()?;
                    self.esperar(Token::CorcheteCierra)?;
                    expr = Expresion::AccesoIndice {
                        objeto: Box::new(expr),
                        indice: Box::new(indice),
                    };
                }
                _ => break,
            }
        }
        Ok(expr)
    }
}
