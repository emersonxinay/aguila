use crate::ast::{
    Captura, Caso, Expresion, Literal, OperadorBinario, OperadorUnario, Parametro, Patron,
    Programa, Sentencia, Token,
};

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

    fn consumir(&mut self, esperado: Token, mensaje: &str) -> Result<(), String> {
        if self.token_actual() == &esperado {
            self.avanzar();
            Ok(())
        } else {
            Err(format!(
                "Error: {}. Se encontró {:?}",
                mensaje,
                self.token_actual()
            ))
        }
    }


    fn consumir_opcional_punto_y_coma(&mut self) {
        if self.token_actual() == &Token::PuntoYComa {
            self.avanzar();
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
            Token::Si => self.parsear_si(),
            Token::Mientras => self.parsear_mientras(),
            Token::Para => self.parsear_para(),
            Token::Funcion => self.parsear_funcion(false, vec![]),
            Token::Asincrono => {
                self.avanzar();
                if self.token_actual() == &Token::Funcion {
                    self.parsear_funcion(true, vec![])
                } else {
                    Err("Se esperaba 'funcion' después de 'asincrono'".to_string())
                }
            }
            Token::Clase => self.parsear_clase(vec![]),
            Token::Retornar => self.parsear_retornar(),
            Token::Romper => {
                self.avanzar();
                self.consumir_opcional_punto_y_coma();
                Ok(Sentencia::Romper)
            }
            Token::Continuar => {
                self.avanzar();
                self.consumir_opcional_punto_y_coma();
                Ok(Sentencia::Continuar)
            }
            Token::Pasar => {
                self.avanzar();
                self.consumir_opcional_punto_y_coma();
                Ok(Sentencia::Pasar)
            }
            Token::Importar => self.parsear_importar(),
            Token::Desde => self.parsear_desde_importar(),
            Token::Intentar => self.parsear_try_catch(),
            Token::Segun => self.parsear_segun(),
            Token::Imprimir => self.parsear_imprimir(),
            Token::Global => self.parsear_global(),
            Token::NoLocal => self.parsear_nolocal(),
            Token::Con => self.parsear_con(),
            Token::Arroba => self.parsear_decorador(),
            Token::Lanzar => self.parsear_lanzar(),
            _ => self.parsear_expresion_sentencia(),
        }
    }

    fn parsear_bloque(&mut self) -> Result<Vec<Sentencia>, String> {
        self.consumir(Token::LlaveAbre, "Se esperaba '{' al inicio del bloque")?;
        let mut sentencias = Vec::new();
        while self.token_actual() != &Token::LlaveCierra && self.token_actual() != &Token::EOF {
            sentencias.push(self.parsear_sentencia()?);
        }
        self.consumir(Token::LlaveCierra, "Se esperaba '}' al final del bloque")?;
        Ok(sentencias)
    }

    fn parsear_si(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // si
        let condicion = self.parsear_expresion()?;
        let entonces = self.parsear_bloque()?;

        let sino = if self.token_actual() == &Token::Sino {
            self.avanzar();
            if self.token_actual() == &Token::Si {
                // sino si -> recursivo
                Some(vec![self.parsear_si()?])
            } else {
                Some(self.parsear_bloque()?)
            }
        } else {
            None
        };

        Ok(Sentencia::Si {
            condicion,
            entonces,
            sino,
        })
    }

    fn parsear_mientras(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // mientras
        let condicion = self.parsear_expresion()?;
        let cuerpo = self.parsear_bloque()?;
        Ok(Sentencia::Mientras { condicion, cuerpo })
    }

    fn parsear_para(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // para
        let variable = match self.avanzar() {
            Token::Identificador(s) => s,
            _ => return Err("Se esperaba identificador en 'para'".to_string()),
        };
        self.consumir(Token::En, "Se esperaba 'en' después de variable en 'para'")?;
        let iterable = self.parsear_expresion()?;
        let cuerpo = self.parsear_bloque()?;
        Ok(Sentencia::Para {
            variable,
            iterable,
            cuerpo,
        })
    }

    fn parsear_funcion(
        &mut self,
        es_async: bool,
        decoradores: Vec<Expresion>,
    ) -> Result<Sentencia, String> {
        if !es_async {
            self.avanzar();
        } // funcion (si ya consumimos asincrono, no avanzamos funcion aqui, pero en el match principal si)
          // Wait, logic in match: if Asincrono -> consume -> check Funcion -> call parsear_funcion(true).
          // Inside parsear_funcion: if !es_async, consume Funcion. If es_async, Funcion already consumed?
          // Let's adjust: match calls parsear_funcion AFTER consuming Token::Funcion.

        let nombre = match self.avanzar() {
            Token::Identificador(s) => s,
            _ => return Err("Se esperaba nombre de función".to_string()),
        };

        self.consumir(Token::ParAbre, "Se esperaba '('")?;
        let mut params = Vec::new();
        if self.token_actual() != &Token::ParCierra {
            loop {
                let param_nombre = match self.avanzar() {
                    Token::Identificador(s) => s,
                    _ => return Err("Se esperaba nombre de parámetro".to_string()),
                };

                let mut tipo = None;
                if self.token_actual() == &Token::DosPuntos {
                    self.avanzar();
                    tipo = Some(self.parsear_tipo()?);
                }

                let mut valor_por_defecto = None;
                if self.token_actual() == &Token::Asignacion {
                    self.avanzar();
                    valor_por_defecto = Some(self.parsear_expresion()?);
                }

                params.push(Parametro {
                    nombre: param_nombre,
                    tipo,
                    valor_por_defecto,
                });

                if self.token_actual() == &Token::Coma {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        self.consumir(Token::ParCierra, "Se esperaba ')'")?;

        // Return type hint?
        if self.token_actual() == &Token::Flecha {
            self.avanzar();
            self.parsear_tipo()?; // Ignoramos el tipo de retorno por ahora en el AST simple
        }

        let cuerpo = self.parsear_bloque()?;
        Ok(Sentencia::Funcion {
            nombre,
            params,
            cuerpo,
            es_async,
            decoradores,
        })
    }

    fn parsear_tipo(&mut self) -> Result<String, String> {
        // Simple type parser: Identificador or Lista[Tipo] etc.
        match self.avanzar() {
            Token::Identificador(s) => Ok(s),
            _ => Err("Se esperaba tipo".to_string()),
        }
    }

    fn parsear_retornar(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // retornar
        let valor = if self.token_actual() != &Token::PuntoYComa {
            Some(self.parsear_expresion()?)
        } else {
            None
        };
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::Retornar(valor))
    }

    fn parsear_imprimir(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // imprimir
        self.consumir(Token::ParAbre, "Se esperaba '('")?;
        let mut args = Vec::new();
        if self.token_actual() != &Token::ParCierra {
            loop {
                args.push(self.parsear_expresion()?);
                if self.token_actual() == &Token::Coma {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        self.consumir(Token::ParCierra, "Se esperaba ')'")?;
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::Imprimir(args))
    }

    fn parsear_expresion_sentencia(&mut self) -> Result<Sentencia, String> {
        let expr = self.parsear_expresion()?;

        if self.token_actual() == &Token::Asignacion {
            self.avanzar();
            let valor = self.parsear_expresion()?;
            self.consumir_opcional_punto_y_coma();
            return Ok(Sentencia::Asignacion {
                objetivo: expr,
                valor,
                tipo: None,
            });
        }

        // Asignación aumentada (+=, -=, etc.)
        if let Some(op) = self.token_a_operador_asignacion(self.token_actual()) {
            self.avanzar();
            let valor = self.parsear_expresion()?;
            self.consumir_opcional_punto_y_coma();
            return Ok(Sentencia::AsignacionAumentada {
                objetivo: expr,
                op,
                valor,
            });
        }

        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::Expresion(expr))
    }

    fn token_a_operador_asignacion(&self, token: &Token) -> Option<OperadorBinario> {
        match token {
            Token::MasIgual => Some(OperadorBinario::Suma),
            Token::MenosIgual => Some(OperadorBinario::Resta),
            Token::PorIgual => Some(OperadorBinario::Multiplicacion),
            Token::DivIgual => Some(OperadorBinario::Division),
            Token::DivEnteraIgual => Some(OperadorBinario::DivisionEntera),
            Token::ModuloIgual => Some(OperadorBinario::Modulo),
            Token::PotenciaIgual => Some(OperadorBinario::Potencia),
            _ => None,
        }
    }

    fn parsear_expresion(&mut self) -> Result<Expresion, String> {
        self.parsear_logica_o()
    }

    fn parsear_logica_o(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_logica_y()?;
        while self.token_actual() == &Token::O {
            self.avanzar();
            let der = self.parsear_logica_y()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op: OperadorBinario::O,
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
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op: OperadorBinario::Y,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_comparacion(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_bitwise_or()?;
        while let Some(op) = self.token_a_operador_comparacion(self.token_actual()) {
            self.avanzar();
            let der = self.parsear_bitwise_or()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn token_a_operador_comparacion(&self, token: &Token) -> Option<OperadorBinario> {
        match token {
            Token::Igual => Some(OperadorBinario::Igual),
            Token::NoIgual => Some(OperadorBinario::NoIgual),
            Token::Mayor => Some(OperadorBinario::Mayor),
            Token::Menor => Some(OperadorBinario::Menor),
            Token::MayorIgual => Some(OperadorBinario::MayorIgual),
            Token::MenorIgual => Some(OperadorBinario::MenorIgual),
            _ => None,
        }
    }

    fn parsear_bitwise_or(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_bitwise_xor()?;
        while self.token_actual() == &Token::Barra {
            self.avanzar();
            let der = self.parsear_bitwise_xor()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op: OperadorBinario::BitOr,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_bitwise_xor(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_bitwise_and()?;
        while self.token_actual() == &Token::Caret {
            self.avanzar();
            let der = self.parsear_bitwise_and()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op: OperadorBinario::BitXor,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_bitwise_and(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_shift()?;
        while self.token_actual() == &Token::Ampersand {
            self.avanzar();
            let der = self.parsear_shift()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op: OperadorBinario::BitAnd,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_shift(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_termino()?;
        while let Some(op) = self.token_a_operador_shift(self.token_actual()) {
            self.avanzar();
            let der = self.parsear_termino()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn token_a_operador_shift(&self, token: &Token) -> Option<OperadorBinario> {
        match token {
            Token::ShiftIzq => Some(OperadorBinario::ShiftIzq),
            Token::ShiftDer => Some(OperadorBinario::ShiftDer),
            _ => None,
        }
    }

    fn parsear_termino(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_factor()?;
        while let Some(op) = self.token_a_operador_termino(self.token_actual()) {
            self.avanzar();
            let der = self.parsear_factor()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn token_a_operador_termino(&self, token: &Token) -> Option<OperadorBinario> {
        match token {
            Token::Mas => Some(OperadorBinario::Suma),
            Token::Menos => Some(OperadorBinario::Resta),
            _ => None,
        }
    }

    fn parsear_factor(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_unario()?;
        while let Some(op) = self.token_a_operador_factor(self.token_actual()) {
            self.avanzar();
            let der = self.parsear_unario()?;
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn token_a_operador_factor(&self, token: &Token) -> Option<OperadorBinario> {
        match token {
            Token::Por => Some(OperadorBinario::Multiplicacion),
            Token::Div => Some(OperadorBinario::Division),
            Token::DivEntera => Some(OperadorBinario::DivisionEntera),
            Token::Modulo => Some(OperadorBinario::Modulo),
            _ => None,
        }
    }

    fn parsear_unario(&mut self) -> Result<Expresion, String> {
        if let Some(op) = self.token_a_operador_unario(self.token_actual()) {
            self.avanzar();
            let exp = self.parsear_unario()?;
            Ok(Expresion::Unaria {
                op,
                exp: Box::new(exp),
            })
        } else {
            self.parsear_potencia()
        }
    }

    fn token_a_operador_unario(&self, token: &Token) -> Option<OperadorUnario> {
        match token {
            Token::Menos => Some(OperadorUnario::Negativo),
            Token::No => Some(OperadorUnario::Not),
            Token::Tilde => Some(OperadorUnario::BitNot),
            _ => None,
        }
    }

    fn parsear_potencia(&mut self) -> Result<Expresion, String> {
        let mut izq = self.parsear_llamada()?;
        if self.token_actual() == &Token::Potencia {
            self.avanzar();
            let der = self.parsear_unario()?; // Right associative
            izq = Expresion::Binaria {
                izq: Box::new(izq),
                op: OperadorBinario::Potencia,
                der: Box::new(der),
            };
        }
        Ok(izq)
    }

    fn parsear_llamada(&mut self) -> Result<Expresion, String> {
        let mut expr = self.parsear_primario()?;

        loop {
            if self.token_actual() == &Token::ParAbre {
                self.avanzar();
                let mut args = Vec::new();
                if self.token_actual() != &Token::ParCierra {
                    loop {
                        args.push(self.parsear_expresion()?);
                        if self.token_actual() == &Token::Coma {
                            self.avanzar();
                        } else {
                            break;
                        }
                    }
                }
                self.consumir(Token::ParCierra, "Se esperaba ')'")?;
                expr = Expresion::Llamada {
                    func: Box::new(expr),
                    args,
                };
            } else if self.token_actual() == &Token::Punto {
                self.avanzar();
                let atributo = match self.avanzar() {
                    Token::Identificador(s) => s,
                    _ => return Err("Se esperaba identificador después de '.'".to_string()),
                };
                expr = Expresion::AccesoAtributo {
                    objeto: Box::new(expr),
                    atributo,
                };
            } else if self.token_actual() == &Token::CorcheteAbre {
                self.avanzar();
                let indice = self.parsear_expresion()?;
                self.consumir(Token::CorcheteCierra, "Se esperaba ']'")?;
                expr = Expresion::AccesoIndice {
                    objeto: Box::new(expr),
                    indice: Box::new(indice),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parsear_primario(&mut self) -> Result<Expresion, String> {
        match self.avanzar() {
            Token::Numero(n) => Ok(Expresion::Literal(Literal::Entero(n as i64))), // Simplificación: todo f64 a i64 si es entero? No, Literal tiene Decimal.
            // TODO: Mejorar Lexer para distinguir int/float o detectar aqui
            Token::Texto(s) => Ok(Expresion::Literal(Literal::Texto(s))),
            Token::TextoInterpolado(s) => {
                // Parsear interpolación es complejo, por ahora lo tratamos como texto
                Ok(Expresion::Literal(Literal::Texto(s)))
            }
            Token::Verdadero => Ok(Expresion::Literal(Literal::Booleano(true))),
            Token::Falso => Ok(Expresion::Literal(Literal::Booleano(false))),
            Token::Nulo => Ok(Expresion::Literal(Literal::Nulo)),
            Token::Identificador(s) => Ok(Expresion::Identificador(s)),
            Token::ParAbre => {
                let expr = self.parsear_expresion()?;
                self.consumir(Token::ParCierra, "Se esperaba ')'")?;
                Ok(expr)
            }
            Token::CorcheteAbre => {
                // Lista
                let mut elementos = Vec::new();
                if self.token_actual() != &Token::CorcheteCierra {
                    loop {
                        elementos.push(self.parsear_expresion()?);
                        if self.token_actual() == &Token::Coma {
                            self.avanzar();
                        } else {
                            break;
                        }
                    }
                }
                self.consumir(Token::CorcheteCierra, "Se esperaba ']'")?;
                Ok(Expresion::Lista(elementos))
            }
            Token::LlaveAbre => {
                // Diccionario o Conjunto
                // Por ahora solo Diccionario vacío o con elementos
                if self.token_actual() == &Token::LlaveCierra {
                    self.avanzar();
                    return Ok(Expresion::Diccionario(vec![]));
                }

                let clave = self.parsear_expresion()?;
                if self.token_actual() == &Token::DosPuntos {
                    // Es diccionario
                    self.avanzar();
                    let valor = self.parsear_expresion()?;
                    let mut pares = vec![(clave, valor)];
                    while self.token_actual() == &Token::Coma {
                        self.avanzar();
                        if self.token_actual() == &Token::LlaveCierra {
                            break;
                        }
                        let k = self.parsear_expresion()?;
                        self.consumir(Token::DosPuntos, "Se esperaba ':'")?;
                        let v = self.parsear_expresion()?;
                        pares.push((k, v));
                    }
                    self.consumir(Token::LlaveCierra, "Se esperaba '}'")?;
                    Ok(Expresion::Diccionario(pares))
                } else {
                    // Es conjunto
                    let mut elementos = vec![clave];
                    while self.token_actual() == &Token::Coma {
                        self.avanzar();
                        if self.token_actual() == &Token::LlaveCierra {
                            break;
                        }
                        elementos.push(self.parsear_expresion()?);
                    }
                    self.consumir(Token::LlaveCierra, "Se esperaba '}'")?;
                    Ok(Expresion::Conjunto(elementos))
                }
            }
            Token::Esperar => {
                let expr = self.parsear_expresion()?;
                Ok(Expresion::Await(Box::new(expr)))
            }
            t => Err(format!("Token inesperado en expresión: {:?}", t)),
        }
    }

    // Stubs for complex statements
    fn parsear_clase(&mut self, _decoradores: Vec<Expresion>) -> Result<Sentencia, String> {
        self.avanzar(); // clase
        let nombre = match self.avanzar() {
            Token::Identificador(s) => s,
            _ => return Err("Se esperaba nombre de clase".to_string()),
        };

        let mut herencia = Vec::new();
        if self.token_actual() == &Token::ParAbre {
            self.avanzar();
            while self.token_actual() != &Token::ParCierra {
                match self.avanzar() {
                    Token::Identificador(s) => herencia.push(s),
                    _ => return Err("Se esperaba identificador en herencia".to_string()),
                }
                if self.token_actual() == &Token::Coma {
                    self.avanzar();
                } else {
                    break;
                }
            }
            self.consumir(Token::ParCierra, "Se esperaba ')'")?;
        }

        let cuerpo = self.parsear_bloque()?;
        Ok(Sentencia::Clase {
            nombre,
            herencia,
            cuerpo,
            decoradores: vec![],
        })
    }

    fn parsear_importar(&mut self) -> Result<Sentencia, String> {
        self.avanzar();
        let modulo = match self.avanzar() {
            Token::Identificador(s) => s,
            _ => return Err("Se esperaba nombre de módulo".to_string()),
        };
        let alias = if self.token_actual() == &Token::Como {
            self.avanzar();
            match self.avanzar() {
                Token::Identificador(s) => Some(s),
                _ => return Err("Se esperaba alias".to_string()),
            }
        } else {
            None
        };
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::Importar { modulo, alias })
    }

    fn parsear_desde_importar(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // desde
        let modulo = match self.avanzar() {
            Token::Identificador(s) => s,
            _ => return Err("Se esperaba nombre de módulo".to_string()),
        };
        self.consumir(Token::Importar, "Se esperaba 'importar'")?;

        let mut elementos = Vec::new();
        loop {
            let nombre = match self.avanzar() {
                Token::Identificador(s) => s,
                Token::Por => "*".to_string(),
                _ => return Err("Se esperaba identificador o '*'".to_string()),
            };

            let alias = if self.token_actual() == &Token::Como {
                self.avanzar();
                match self.avanzar() {
                    Token::Identificador(s) => Some(s),
                    _ => return Err("Se esperaba alias".to_string()),
                }
            } else {
                None
            };

            elementos.push((nombre, alias));

            if self.token_actual() == &Token::Coma {
                self.avanzar();
            } else {
                break;
            }
        }
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::DesdeImportar { modulo, elementos })
    }

    fn parsear_try_catch(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // intentar
        let cuerpo = self.parsear_bloque()?;
        let mut capturas = Vec::new();

        while self.token_actual() == &Token::Capturar {
            self.avanzar();
            let mut tipo = None;
            let mut variable = None;

            if self.token_actual() != &Token::LlaveAbre {
                tipo = match self.avanzar() {
                    Token::Identificador(s) => Some(s),
                    _ => return Err("Se esperaba tipo de excepción".to_string()),
                };

                if self.token_actual() == &Token::Como {
                    self.avanzar();
                    variable = match self.avanzar() {
                        Token::Identificador(s) => Some(s),
                        _ => return Err("Se esperaba nombre de variable".to_string()),
                    };
                }
            }

            let bloque_catch = self.parsear_bloque()?;
            capturas.push(Captura {
                tipo,
                variable,
                cuerpo: bloque_catch,
            });
        }

        let finalmente = if self.token_actual() == &Token::Finalmente {
            self.avanzar();
            Some(self.parsear_bloque()?)
        } else {
            None
        };

        Ok(Sentencia::TryCatch {
            cuerpo,
            capturas,
            finalmente,
        })
    }

    fn parsear_segun(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // segun
        let valor = self.parsear_expresion()?;
        self.consumir(Token::LlaveAbre, "Se esperaba '{'")?;

        let mut casos = Vec::new();
        let mut defecto = None;

        while self.token_actual() != &Token::LlaveCierra {
            if self.token_actual() == &Token::Caso {
                self.avanzar();
                let patron = self.parsear_patron()?;
                self.consumir(Token::DosPuntos, "Se esperaba ':'")?; // Opcional en Python pero útil
                                                                     // Ojo: Python usa 'case pattern:'
                let cuerpo = self.parsear_bloque()?; // O sentencias hasta el siguiente caso? Usaremos bloques {} por ahora para simplificar
                casos.push(Caso {
                    patron,
                    guarda: None,
                    cuerpo,
                });
            } else if self.token_actual() == &Token::Defecto {
                self.avanzar();
                self.consumir(Token::DosPuntos, "Se esperaba ':'")?;
                defecto = Some(self.parsear_bloque()?);
            } else {
                return Err("Se esperaba 'caso' o 'defecto'".to_string());
            }
        }
        self.consumir(Token::LlaveCierra, "Se esperaba '}'")?;
        Ok(Sentencia::Segun {
            valor,
            casos,
            defecto,
        })
    }

    fn parsear_patron(&mut self) -> Result<Patron, String> {
        // Simplificado
        match self.avanzar() {
            Token::Numero(n) => Ok(Patron::Literal(Literal::Entero(n as i64))),
            Token::Texto(s) => Ok(Patron::Literal(Literal::Texto(s))),
            Token::Identificador(s) => {
                if s == "_" {
                    Ok(Patron::Comodin)
                } else {
                    Ok(Patron::Identificador(s))
                }
            }
            _ => Err("Patrón no soportado".to_string()),
        }
    }

    fn parsear_global(&mut self) -> Result<Sentencia, String> {
        self.avanzar();
        let mut vars = Vec::new();
        loop {
            match self.avanzar() {
                Token::Identificador(s) => vars.push(s),
                _ => return Err("Se esperaba identificador".to_string()),
            }
            if self.token_actual() == &Token::Coma {
                self.avanzar();
            } else {
                break;
            }
        }
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::Global(vars))
    }

    fn parsear_nolocal(&mut self) -> Result<Sentencia, String> {
        self.avanzar();
        let mut vars = Vec::new();
        loop {
            match self.avanzar() {
                Token::Identificador(s) => vars.push(s),
                _ => return Err("Se esperaba identificador".to_string()),
            }
            if self.token_actual() == &Token::Coma {
                self.avanzar();
            } else {
                break;
            }
        }
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::NoLocal(vars))
    }

    fn parsear_con(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // con
        let expr = self.parsear_expresion()?;
        let alias = if self.token_actual() == &Token::Como {
            self.avanzar();
            match self.avanzar() {
                Token::Identificador(s) => Some(s),
                _ => return Err("Se esperaba alias".to_string()),
            }
        } else {
            None
        };
        let cuerpo = self.parsear_bloque()?;
        Ok(Sentencia::Con {
            items: vec![(expr, alias)],
            cuerpo,
        })
    }

    fn parsear_decorador(&mut self) -> Result<Sentencia, String> {
        self.avanzar(); // @
        let expr = self.parsear_expresion()?; // Decorador puede ser llamada
                                              // El decorador debe ir seguido de una función o clase
                                              // Recursivamente parseamos la siguiente sentencia y le inyectamos el decorador
                                              // Esto es un poco hacky, idealmente acumulamos decoradores.
                                              // Simplificación: Solo soportamos 1 decorador por ahora o requerimos estructura diferente
                                              // Mejor: parsear_sentencia debería manejar decoradores acumulados.
                                              // Por ahora devolvemos error o hack.
                                              // Hack: Llamar a parsear_funcion pasando el decorador.
        if self.token_actual() == &Token::Funcion {
            self.parsear_funcion(false, vec![expr])
        } else if self.token_actual() == &Token::Clase {
            self.parsear_clase(vec![expr])
        } else if self.token_actual() == &Token::Asincrono {
            self.avanzar();
            self.parsear_funcion(true, vec![expr])
        } else {
            Err("Decorador debe preceder a función o clase".to_string())
        }
    }

    fn parsear_lanzar(&mut self) -> Result<Sentencia, String> {
        self.avanzar();
        let expr = self.parsear_expresion()?;
        self.consumir_opcional_punto_y_coma();
        Ok(Sentencia::Lanzar(expr))
    }
}
