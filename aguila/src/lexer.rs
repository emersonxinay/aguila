use crate::ast::Token;

pub struct Lexer {
    entrada: Vec<char>,
    posicion: usize,
    linea: usize,
    columna: usize,
}

impl Lexer {
    pub fn nuevo(entrada: &str) -> Self {
        Lexer {
            entrada: entrada.chars().collect(),
            posicion: 0,
            linea: 1,
            columna: 1,
        }
    }

    fn car_actual(&self) -> Option<char> {
        if self.posicion < self.entrada.len() {
            Some(self.entrada[self.posicion])
        } else {
            None
        }
    }

    fn car_siguiente(&self) -> Option<char> {
        if self.posicion + 1 < self.entrada.len() {
            Some(self.entrada[self.posicion + 1])
        } else {
            None
        }
    }

    fn avanzar(&mut self) -> Option<char> {
        if let Some(car) = self.car_actual() {
            self.posicion += 1;
            if car == '\n' {
                self.linea += 1;
                self.columna = 1;
            } else {
                self.columna += 1;
            }
            Some(car)
        } else {
            None
        }
    }

    fn omitir_espacios(&mut self) {
        while let Some(car) = self.car_actual() {
            if car.is_whitespace() {
                self.avanzar();
            } else {
                break;
            }
        }
    }

    fn omitir_resto_linea(&mut self) {
        while let Some(car) = self.car_actual() {
            if car == '\n' {
                break;
            }
            self.avanzar();
        }
    }

    fn leer_numero(&mut self) -> Token {
        let mut numero = String::new();
        let mut tiene_punto = false;

        while let Some(car) = self.car_actual() {
            if car.is_numeric() {
                numero.push(car);
                self.avanzar();
            } else if car == '.' {
                if !tiene_punto {
                    if let Some(siguiente) = self.car_siguiente() {
                        if siguiente.is_numeric() {
                            tiene_punto = true;
                            numero.push(car);
                            self.avanzar();
                            continue;
                        }
                    }
                }
                break;
            } else {
                break;
            }
        }

        if let Ok(num) = numero.parse::<f64>() {
            Token::Numero(num)
        } else {
            Token::Numero(0.0)
        }
    }

    fn leer_texto(&mut self, delimitador: char) -> Token {
        self.avanzar(); // omitir comilla inicial
        let mut texto = String::new();
        while let Some(car) = self.car_actual() {
            if car == delimitador {
                self.avanzar();
                break;
            }
            if car == '\\' {
                self.avanzar();
                if let Some(siguiente) = self.car_actual() {
                    match siguiente {
                        'n' => texto.push('\n'),
                        't' => texto.push('\t'),
                        'r' => texto.push('\r'),
                        '\\' => texto.push('\\'),
                        '"' => texto.push('"'),
                        '\'' => texto.push('\''),
                        _ => {
                            texto.push('\\');
                            texto.push(siguiente);
                        }
                    }
                    self.avanzar();
                }
            } else {
                texto.push(car);
                self.avanzar();
            }
        }
        Token::Texto(texto)
    }

    fn leer_texto_interpolado(&mut self, delimitador: char) -> String {
        let mut texto = String::new();
        while let Some(car) = self.car_actual() {
            if car == delimitador {
                self.avanzar();
                break;
            }
            if car == '\\' {
                self.avanzar();
                if let Some(siguiente) = self.car_actual() {
                    match siguiente {
                        'n' => texto.push('\n'),
                        't' => texto.push('\t'),
                        'r' => texto.push('\r'),
                        '\\' => texto.push('\\'),
                        '"' => texto.push('"'),
                        _ => {
                            texto.push('\\');
                            texto.push(siguiente);
                        }
                    }
                    self.avanzar();
                }
            } else {
                texto.push(car);
                self.avanzar();
            }
        }
        texto
    }

    fn leer_identificador(&mut self) -> String {
        let mut ident = String::new();
        while let Some(car) = self.car_actual() {
            if car.is_alphanumeric() || car == '_' {
                ident.push(car);
                self.avanzar();
            } else {
                break;
            }
        }
        ident
    }

    pub fn siguiente_token(&mut self) -> Token {
        loop {
            self.omitir_espacios();

            if self.car_actual() == Some('#') {
                self.omitir_resto_linea();
                continue;
            }
            break;
        }

        match self.car_actual() {
            None => Token::EOF,
            Some('+') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::MasIgual
                } else {
                    Token::Mas
                }
            }
            Some('-') => {
                self.avanzar();
                if self.car_actual() == Some('>') {
                    self.avanzar();
                    Token::Flecha
                } else if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::MenosIgual
                } else {
                    Token::Menos
                }
            }
            Some('*') => {
                self.avanzar();
                if self.car_actual() == Some('*') {
                    self.avanzar();
                    if self.car_actual() == Some('=') {
                        self.avanzar();
                        Token::PotenciaIgual
                    } else {
                        Token::Potencia
                    }
                } else if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::PorIgual
                } else {
                    Token::Por
                }
            }
            Some('/') => {
                self.avanzar();
                if self.car_actual() == Some('/') {
                    self.avanzar();
                    if self.car_actual() == Some('=') {
                        self.avanzar();
                        Token::DivEnteraIgual
                    } else {
                        Token::DivEntera
                    }
                } else if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::DivIgual
                } else {
                    Token::Div
                }
            }
            Some('%') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::ModuloIgual
                } else {
                    Token::Modulo
                }
            }
            Some('(') => {
                self.avanzar();
                Token::ParAbre
            }
            Some(')') => {
                self.avanzar();
                Token::ParCierra
            }
            Some('{') => {
                self.avanzar();
                Token::LlaveAbre
            }
            Some('}') => {
                self.avanzar();
                Token::LlaveCierra
            }
            Some('[') => {
                self.avanzar();
                Token::CorcheteAbre
            }
            Some(']') => {
                self.avanzar();
                Token::CorcheteCierra
            }
            Some(',') => {
                self.avanzar();
                Token::Coma
            }
            Some('.') => {
                self.avanzar();
                Token::Punto
            }
            Some(':') => {
                self.avanzar();
                Token::DosPuntos
            }
            Some(';') => {
                self.avanzar();
                Token::PuntoYComa
            }
            Some('@') => {
                self.avanzar();
                Token::Arroba
            }
            Some('&') => {
                self.avanzar();
                Token::Ampersand
            }
            Some('|') => {
                self.avanzar();
                Token::Barra
            }
            Some('^') => {
                self.avanzar();
                Token::Caret
            }
            Some('~') => {
                self.avanzar();
                Token::Tilde
            }
            Some('=') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::Igual
                } else {
                    Token::Asignacion
                }
            }
            Some('>') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::MayorIgual
                } else if self.car_actual() == Some('>') {
                    self.avanzar();
                    Token::ShiftDer
                } else {
                    Token::Mayor
                }
            }
            Some('<') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::MenorIgual
                } else if self.car_actual() == Some('<') {
                    self.avanzar();
                    Token::ShiftIzq
                } else {
                    Token::Menor
                }
            }
            Some('!') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::NoIgual
                } else {
                    Token::Error(
                        "Caracter '!' inesperado. Usar 'no' para negación lógica.".to_string(),
                    )
                }
            }
            Some('"') => self.leer_texto('"'),
            Some('\'') => self.leer_texto('\''),
            Some(car) if car.is_numeric() => self.leer_numero(),
            Some(car) if car.is_alphabetic() || car == '_' => {
                let ident = self.leer_identificador();
                if ident == "a"
                    && (self.car_actual() == Some('"') || self.car_actual() == Some('\''))
                {
                    let delim = self.car_actual().unwrap();
                    self.avanzar(); // Consumir comilla
                    let texto = self.leer_texto_interpolado(delim);
                    Token::TextoInterpolado(texto)
                } else {
                    match ident.as_str() {
                        "si" => Token::Si,
                        "sino" => Token::Sino,
                        "mientras" => Token::Mientras,
                        "para" => Token::Para,
                        "en" => Token::En,
                        "romper" => Token::Romper,
                        "continuar" => Token::Continuar,
                        "retornar" => Token::Retornar,
                        "funcion" => Token::Funcion,
                        "clase" => Token::Clase,
                        "verdadero" => Token::Verdadero,
                        "falso" => Token::Falso,
                        "nulo" => Token::Nulo,
                        "y" => Token::Y,
                        "o" => Token::O,
                        "no" => Token::No,
                        "importar" => Token::Importar,
                        "desde" => Token::Desde,
                        "como" => Token::Como,
                        "intentar" => Token::Intentar,
                        "capturar" => Token::Capturar,
                        "finalmente" => Token::Finalmente,
                        "lanzar" => Token::Lanzar,
                        "global" => Token::Global,
                        "nolocal" => Token::NoLocal,
                        "pasar" => Token::Pasar,
                        "eliminar" => Token::Eliminar,
                        "con" => Token::Con,
                        "asincrono" => Token::Asincrono,
                        "esperar" => Token::Esperar,
                        "ceder" => Token::Ceder,
                        "segun" => Token::Segun,
                        "caso" => Token::Caso,
                        "defecto" => Token::Defecto,
                        "imprime" | "imprimir" => Token::Imprimir,
                        "afirmar" => Token::Afirmar,
                        "nuevo" => Token::Nuevo,
                        "let" => Token::Let,
                        _ => Token::Identificador(ident),
                    }
                }
            }
            Some(car) => {
                self.avanzar();
                Token::Error(format!("Carácter inesperado: '{}'", car))
            }
        }
    }

    pub fn tokenizar(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.siguiente_token();
            if token == Token::EOF {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
