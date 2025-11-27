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
                    // Solo consumir el punto si el siguiente carácter es un dígito
                    // Esto permite que "10.metodo()" se parsee como Numero(10), Punto, Ident(metodo)
                    // Y que "20.4.5" se parsee como Numero(20.4), Punto, Numero(5)
                    if let Some(siguiente) = self.car_siguiente() {
                        if siguiente.is_numeric() {
                            tiene_punto = true;
                            numero.push(car);
                            self.avanzar();
                            continue;
                        }
                    }
                }
                // Si ya tiene punto o el siguiente no es dígito, terminamos el número
                break;
            } else {
                break;
            }
        }
        
        // Parsear el número acumulado
        if let Ok(num) = numero.parse::<f64>() {
            Token::Numero(num)
        } else {
            // Fallback por si acaso, aunque con la lógica anterior debería ser siempre válido
            Token::Numero(0.0)
        }
    }

    fn leer_texto(&mut self, delimitador: char) -> Token {
        self.avanzar(); // omitir comilla
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
        Token::Texto(texto)
    }

    // Nueva función para leer texto interpolado
    fn leer_texto_interpolado(&mut self, delimitador: char) -> String {
        // Asumimos que 'a' y el delimitador inicial ya fueron consumidos
        let mut texto = String::new();
        while let Some(car) = self.car_actual() {
            if car == delimitador {
                self.avanzar();
                break;
            }
            // Aquí se añadiría la lógica para manejar las interpolaciones,
            // por ejemplo, ${expresion}. Por ahora, se comporta como leer_texto.
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
            
            // Verificación anticipada de // para comentarios ELIMINADA para soportar división entera
            // if self.car_actual() == Some('/') && self.car_siguiente() == Some('/') {
            //     self.omitir_resto_linea();
            //     continue;
            // }

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
                    Token::Potencia
                } else {
                    Token::Por
                }
            }
            Some('/') => {
                self.avanzar();
                if self.car_actual() == Some('/') {
                    self.avanzar();
                    Token::DivEntera
                } else {
                    Token::Div
                }
            }
            Some('%') => {
                self.avanzar();
                Token::Modulo
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
                } else {
                    Token::Mayor
                }
            }
            Some('<') => {
                self.avanzar();
                if self.car_actual() == Some('=') {
                    self.avanzar();
                    Token::MenorIgual
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
                    panic!("Error léxico en línea {}: '!' inesperado", self.linea);
                }
            }
            Some('"') => self.leer_texto('"'),
            Some('\'') => self.leer_texto('\''),
            Some(car) if car.is_numeric() => self.leer_numero(),
            Some(car) if car.is_alphabetic() || car == '_' => {
                let ident = self.leer_identificador();
                if ident == "a" && self.car_actual() == Some('"') {
                    self.avanzar(); // Consumir "
                    let texto = self.leer_texto_interpolado('"');
                    Token::TextoInterpolado(texto)
                } else {
                    match ident.as_str() {
                        "funcion" => Token::Funcion,
                        "si" => Token::Si,
                        "sino" => Token::Sino,
                        "mientras" => Token::Mientras,
                        "para" => Token::Para,
                        "en" => Token::En,
                        "hasta" => Token::Hasta,
                        "clase" => Token::Clase,
                        "imprimir" => Token::Imprimir,
                        "verdadero" => Token::Verdadero,
                        "falso" => Token::Falso,
                        "nulo" => Token::Nulo,
                        "importar" => Token::Importar,
                        "retornar" => Token::Retornar,
                        "intentar" => Token::Intentar,
                        "capturar" => Token::Capturar,
                        "nuevo" => Token::Nuevo,
                        "asincrono" => Token::Asincrono,
                        "esperar" => Token::Esperar,
                        "segun" => Token::Segun,
                        "caso" => Token::Caso,
                        "defecto" => Token::Defecto,
                        "y" => Token::Y,
                        "o" => Token::O,
                        "no" => Token::No,
                        _ => Token::Identificador(ident),
                    }
                }
            }
            Some(car) => {
                panic!(
                    "Error léxico en línea {}, columna {}: carácter '{}' inesperado",
                    self.linea, self.columna, car
                );
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
