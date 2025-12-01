use crate::ast::{Expresion, Programa, Sentencia};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::{NativeFn, Value};
use async_recursion::async_recursion;
use futures::future::FutureExt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::rc::Rc;

#[derive(Clone)]
pub struct Interprete {
    variables: Vec<Rc<RefCell<HashMap<String, Value>>>>,
    funciones: HashMap<
        String,
        (
            Vec<(String, Option<String>)>,
            Option<String>,
            Vec<Sentencia>,
            bool,
        ),
    >,
    clases: HashMap<
        String,
        (
            Option<String>,
            Vec<(String, Option<String>)>,
            Vec<(String, Vec<(String, Option<String>)>, Vec<Sentencia>)>,
        ),
    >,
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
        interprete.registrar_modulo_db();
        interprete.registrar_modulo_util();
        interprete
    }

    fn registrar_funciones_nativas(&mut self) {
        let mut fs_metodos = HashMap::new();

        // leer(mensaje_opcional)
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert(
                "leer".to_string(),
                Value::FuncionNativa(NativeFn(Rc::new(|args| {
                    // 1. Mostrar mensaje si existe
                    if let Some(msg) = args.get(0) {
                        print!("{}", msg.a_texto());
                        let _ = std::io::stdout().flush();
                    }

                    // 2. Leer entrada
                    let mut buffer = String::new();
                    match std::io::stdin().read_line(&mut buffer) {
                        Ok(_) => {
                            let input = buffer.trim();

                            // 3. Inferencia de tipos

                            // Booleano
                            if input.eq_ignore_ascii_case("verdadero") {
                                return Ok(Value::Logico(true));
                            }
                            if input.eq_ignore_ascii_case("falso") {
                                return Ok(Value::Logico(false));
                            }

                            // Número
                            if let Ok(num) = input.parse::<f64>() {
                                return Ok(Value::Numero(num));
                            }

                            // Texto (por defecto)
                            Ok(Value::Texto(input.to_string()))
                        }
                        Err(e) => Err(format!("Error al leer entrada: {}", e)),
                    }
                }))),
            );
        }

        // fs.leer(ruta)
        fs_metodos.insert(
            "leer".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 1 {
                    return Err("fs.leer espera 1 argumento (ruta)".to_string());
                }
                if let Value::Texto(ruta) = &args[0] {
                    match std::fs::read_to_string(ruta) {
                        Ok(contenido) => Ok(Value::Texto(contenido)),
                        Err(e) => Err(format!("Error al leer archivo: {}", e)),
                    }
                } else {
                    Err("El argumento de fs.leer debe ser texto".to_string())
                }
            }))),
        );

        // fs.escribir(ruta, contenido)
        fs_metodos.insert(
            "escribir".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("fs.escribir espera 2 argumentos (ruta, contenido)".to_string());
                }
                let ruta = if let Value::Texto(s) = &args[0] {
                    s
                } else {
                    return Err("Arg 1 debe ser texto".to_string());
                };
                let contenido = match &args[1] {
                    Value::Texto(s) => s.clone(),
                    Value::Numero(n) => n.to_string(),
                    _ => args[1].a_texto(),
                };

                match std::fs::write(ruta, contenido) {
                    Ok(_) => Ok(Value::Nulo),
                    Err(e) => Err(format!("Error al escribir archivo: {}", e)),
                }
            }))),
        );

        // afirmar(condicion, mensaje_opcional)
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert(
                "afirmar".to_string(),
                Value::FuncionNativa(NativeFn(Rc::new(|args| {
                    if args.is_empty() {
                        return Err("afirmar espera al menos 1 argumento (condicion)".to_string());
                    }

                    let condicion = args[0].a_booleano();
                    if !condicion {
                        let mensaje = if args.len() > 1 {
                            args[1].a_texto()
                        } else {
                            "Afirmación fallida".to_string()
                        };
                        return Err(format!("Error de Aserción: {}", mensaje));
                    }

                    Ok(Value::Nulo)
                }))),
            );
        }

        // conjunto(lista_opcional)
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert(
                "conjunto".to_string(),
                Value::FuncionNativa(NativeFn(Rc::new(|args| {
                    let mut set = std::collections::HashSet::new();

                    if !args.is_empty() {
                        if let Value::Lista(lista) = &args[0] {
                            for item in lista.borrow().iter() {
                                set.insert(item.clone());
                            }
                        } else {
                            return Err(
                                "conjunto() espera una lista como argumento opcional".to_string()
                            );
                        }
                    }

                    Ok(Value::Conjunto(Rc::new(RefCell::new(set))))
                }))),
            );
        }

        let fs_modulo = Value::Diccionario(Rc::new(RefCell::new(fs_metodos)));

        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("fs".to_string(), fs_modulo);
        }

        // --- Módulo JSON ---
        let mut json_metodos = HashMap::new();

        // json.parsear(texto)
        json_metodos.insert(
            "parsear".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 1 {
                    return Err("json.parsear espera 1 argumento (texto)".to_string());
                }
                if let Value::Texto(json_str) = &args[0] {
                    match serde_json::from_str::<serde_json::Value>(json_str) {
                        Ok(v) => Ok(json_to_value(&v)),
                        Err(e) => Err(format!("Error al parsear JSON: {}", e)),
                    }
                } else {
                    Err("El argumento de json.parsear debe ser texto".to_string())
                }
            }))),
        );

        // json.stringificar(valor)
        json_metodos.insert(
            "stringificar".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 1 {
                    return Err("json.stringificar espera 1 argumento (valor)".to_string());
                }
                let v = value_to_json(&args[0]);
                match serde_json::to_string_pretty(&v) {
                    Ok(s) => Ok(Value::Texto(s)),
                    Err(e) => Err(format!("Error al stringificar JSON: {}", e)),
                }
            }))),
        );

        let json_modulo = Value::Diccionario(Rc::new(RefCell::new(json_metodos)));

        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("json".to_string(), json_modulo);
        }

        // --- Módulo MATE (Math) ---
        let mut mate_metodos = HashMap::new();

        mate_metodos.insert("pi".to_string(), Value::Numero(std::f64::consts::PI));

        mate_metodos.insert(
            "sin".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.sin()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "cos".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.cos()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "tan".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.tan()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "raiz".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.sqrt()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "potencia".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("potencia espera 2 argumentos".to_string());
                }
                match (&args[0], &args[1]) {
                    (Value::Numero(b), Value::Numero(e)) => Ok(Value::Numero(b.powf(*e))),
                    _ => Err("Args deben ser números".to_string()),
                }
            }))),
        );
        mate_metodos.insert(
            "abs".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.abs()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "piso".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.floor()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "techo".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.ceil()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "redondear".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if let Some(Value::Numero(n)) = args.get(0) {
                    Ok(Value::Numero(n.round()))
                } else {
                    Err("Arg debe ser número".to_string())
                }
            }))),
        );
        mate_metodos.insert(
            "aleatorio".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|_| Ok(Value::Numero(rand::random()))))),
        );

        let mate_modulo = Value::Diccionario(Rc::new(RefCell::new(mate_metodos)));
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("mate".to_string(), mate_modulo);
        }

        // --- Módulo FECHA ---
        let mut fecha_metodos = HashMap::new();

        fecha_metodos.insert(
            "ahora".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|_| {
                let now = chrono::Utc::now();
                Ok(Value::Numero(now.timestamp_millis() as f64))
            }))),
        );

        fecha_metodos.insert(
            "formato".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("formato espera 2 argumentos (timestamp, fmt)".to_string());
                }
                let ts = if let Value::Numero(n) = args[0] {
                    n as i64
                } else {
                    return Err("Arg 1 debe ser timestamp (numero)".to_string());
                };
                let fmt = if let Value::Texto(s) = &args[1] {
                    s
                } else {
                    return Err("Arg 2 debe ser formato (texto)".to_string());
                };

                if let Some(dt) = chrono::DateTime::from_timestamp_millis(ts) {
                    Ok(Value::Texto(dt.format(fmt).to_string()))
                } else {
                    Err("Timestamp inválido".to_string())
                }
            }))),
        );

        let fecha_modulo = Value::Diccionario(Rc::new(RefCell::new(fecha_metodos)));
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("fecha".to_string(), fecha_modulo);
        }

        // --- Módulo RED ---
        let mut red_metodos = HashMap::new();

        // red.servidor(puerto)
        red_metodos.insert(
            "servidor".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 1 {
                    return Err("red.servidor espera 1 argumento (puerto)".to_string());
                }
                let puerto = match &args[0] {
                    Value::Numero(n) => *n as u16,
                    _ => return Err("Puerto debe ser número".to_string()),
                };

                let listener = match TcpListener::bind(format!("0.0.0.0:{}", puerto)) {
                    Ok(l) => Rc::new(l),
                    Err(e) => return Err(format!("Error al iniciar servidor: {}", e)),
                };

                let mut servidor_obj = HashMap::new();

                // servidor.aceptar()
                let listener_clone = listener.clone();
                servidor_obj.insert(
                    "aceptar".to_string(),
                    Value::FuncionNativa(NativeFn(Rc::new(move |_| {
                        match listener_clone.accept() {
                            Ok((stream, addr)) => {
                                println!("Conexión aceptada de: {}", addr);
                                // let stream_clone =
                                //     stream.try_clone().expect("Error al clonar stream");
                                let stream = Rc::new(RefCell::new(stream));

                                let mut cliente_obj = HashMap::new();

                                // cliente.leer()
                                let stream_leer = stream.clone();
                                cliente_obj.insert(
                                    "leer".to_string(),
                                    Value::FuncionNativa(NativeFn(Rc::new(move |_| {
                                        let mut buffer = [0; 1024];
                                        match stream_leer.borrow_mut().read(&mut buffer) {
                                            Ok(n) => {
                                                let s = String::from_utf8_lossy(&buffer[..n])
                                                    .to_string();
                                                Ok(Value::Texto(s))
                                            }
                                            Err(e) => {
                                                Err(format!("Error al leer del socket: {}", e))
                                            }
                                        }
                                    }))),
                                );

                                // cliente.escribir(texto)
                                let stream_escribir = stream.clone();
                                cliente_obj.insert(
                                    "escribir".to_string(),
                                    Value::FuncionNativa(NativeFn(Rc::new(move |args| {
                                        if args.len() != 1 {
                                            return Err(
                                                "cliente.escribir espera 1 argumento (texto)"
                                                    .to_string(),
                                            );
                                        }
                                        let texto = args[0].a_texto();
                                        match stream_escribir.borrow_mut().write(texto.as_bytes()) {
                                            Ok(_) => Ok(Value::Logico(true)),
                                            Err(e) => {
                                                Err(format!("Error al escribir en socket: {}", e))
                                            }
                                        }
                                    }))),
                                );

                                // cliente.cerrar()
                                // En Rust, el socket se cierra cuando se dropea el TcpStream (Rc count llega a 0).
                                // Podemos forzarlo o dejar que el GC de ÁGUILA lo maneje.
                                // Por ahora, explícito no es necesario, pero útil para API.
                                cliente_obj.insert(
                                    "cerrar".to_string(),
                                    Value::FuncionNativa(NativeFn(Rc::new(|_| {
                                        Ok(Value::Logico(true))
                                    }))),
                                );

                                Ok(Value::Instancia {
                                    clase: "ClienteTCP".to_string(),
                                    atributos: Rc::new(RefCell::new(cliente_obj)),
                                })
                            }
                            Err(e) => Err(format!("Error al aceptar conexión: {}", e)),
                        }
                    }))),
                );

                Ok(Value::Instancia {
                    clase: "ServidorTCP".to_string(),
                    atributos: Rc::new(RefCell::new(servidor_obj)),
                })
            }))),
        );

        // red.conectar(host, puerto)
        red_metodos.insert(
            "conectar".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("red.conectar espera 2 argumentos (host, puerto)".to_string());
                }
                let host = args[0].a_texto();
                let puerto = match &args[1] {
                    Value::Numero(n) => *n as u16,
                    _ => return Err("Puerto debe ser número".to_string()),
                };

                match TcpStream::connect(format!("{}:{}", host, puerto)) {
                    Ok(stream) => {
                        let stream = Rc::new(RefCell::new(stream));
                        let mut cliente_obj = HashMap::new();

                        // cliente.leer()
                        let stream_leer = stream.clone();
                        cliente_obj.insert(
                            "leer".to_string(),
                            Value::FuncionNativa(NativeFn(Rc::new(move |_| {
                                let mut buffer = [0; 1024];
                                match stream_leer.borrow_mut().read(&mut buffer) {
                                    Ok(n) => {
                                        let s = String::from_utf8_lossy(&buffer[..n]).to_string();
                                        Ok(Value::Texto(s))
                                    }
                                    Err(e) => Err(format!("Error al leer del socket: {}", e)),
                                }
                            }))),
                        );

                        // cliente.escribir(texto)
                        let stream_escribir = stream.clone();
                        cliente_obj.insert(
                            "escribir".to_string(),
                            Value::FuncionNativa(NativeFn(Rc::new(move |args| {
                                if args.len() != 1 {
                                    return Err(
                                        "cliente.escribir espera 1 argumento (texto)".to_string()
                                    );
                                }
                                let texto = args[0].a_texto();
                                match stream_escribir.borrow_mut().write(texto.as_bytes()) {
                                    Ok(_) => Ok(Value::Logico(true)),
                                    Err(e) => Err(format!("Error al escribir en socket: {}", e)),
                                }
                            }))),
                        );

                        // cliente.cerrar()
                        // cliente.cerrar()
                        let stream_cerrar = stream.clone();
                        cliente_obj.insert(
                            "cerrar".to_string(),
                            Value::FuncionNativa(NativeFn(Rc::new(move |_| {
                                match stream_cerrar
                                    .borrow_mut()
                                    .shutdown(std::net::Shutdown::Both)
                                {
                                    Ok(_) => Ok(Value::Logico(true)),
                                    Err(e) => Err(format!("Error al cerrar conexión: {}", e)),
                                }
                            }))),
                        );

                        Ok(Value::Instancia {
                            clase: "ClienteTCP".to_string(),
                            atributos: Rc::new(RefCell::new(cliente_obj)),
                        })
                    }
                    Err(e) => Err(format!("Error al conectar: {}", e)),
                }
            }))),
        );

        let red_modulo = Value::Diccionario(Rc::new(RefCell::new(red_metodos)));

        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("red".to_string(), red_modulo);
        }

        // --- Módulo CADENA ---
        let mut cadena_metodos = HashMap::new();

        // cadena.dividir(texto, separador)
        cadena_metodos.insert(
            "dividir".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("cadena.dividir espera 2 argumentos (texto, separador)".to_string());
                }
                let texto = if let Value::Texto(s) = &args[0] {
                    s
                } else {
                    return Err("Arg 1 debe ser texto".to_string());
                };
                let separador = if let Value::Texto(s) = &args[1] {
                    s
                } else {
                    return Err("Arg 2 debe ser texto".to_string());
                };

                let partes: Vec<Value> = texto
                    .split(separador)
                    .map(|s| Value::Texto(s.to_string()))
                    .collect();

                Ok(Value::Lista(Rc::new(RefCell::new(partes))))
            }))),
        );

        // cadena.contiene(texto, subtexto)
        cadena_metodos.insert(
            "contiene".to_string(),
            Value::FuncionNativa(NativeFn(Rc::new(|args| {
                if args.len() != 2 {
                    return Err("cadena.contiene espera 2 argumentos (texto, subtexto)".to_string());
                }
                let texto = if let Value::Texto(s) = &args[0] {
                    s
                } else {
                    return Err("Arg 1 debe ser texto".to_string());
                };
                let subtexto = if let Value::Texto(s) = &args[1] {
                    s
                } else {
                    return Err("Arg 2 debe ser texto".to_string());
                };

                Ok(Value::Logico(texto.contains(subtexto)))
            }))),
        );

        let cadena_modulo = Value::Diccionario(Rc::new(RefCell::new(cadena_metodos)));

        if let Some(scope) = self.variables.first_mut() {
            scope
                .borrow_mut()
                .insert("cadena".to_string(), cadena_modulo);
        }
    }

    pub async fn ejecutar(&mut self, programa: Programa) -> Result<Option<Value>, String> {
        for sent in programa.sentencias {
            if let Some(val) = self.ejecutar_sentencia(&sent).await? {
                return Ok(Some(val));
            }
        }
        Ok(None)
    }

    #[async_recursion(?Send)]
    async fn ejecutar_sentencia(&mut self, sentencia: &Sentencia) -> Result<Option<Value>, String> {
        match sentencia {
            Sentencia::Asignacion {
                nombre,
                tipo: _,
                valor,
            } => {
                let val = self.evaluar_expresion(valor).await?;
                if let Some(scope) = self.variables.last_mut() {
                    scope.borrow_mut().insert(nombre.clone(), val.clone());
                }
                Ok(None)
            }
            Sentencia::AsignacionIndice {
                objeto,
                indice,
                valor,
            } => {
                let obj_val = self.evaluar_expresion(objeto).await?;
                let idx_val = self.evaluar_expresion(indice).await?;
                let new_val = self.evaluar_expresion(valor).await?;

                match obj_val {
                    Value::Lista(lista) => {
                        if let Value::Numero(idx) = idx_val {
                            let idx_usize = idx as usize;
                            let mut list_mut = lista.borrow_mut();
                            if idx_usize < list_mut.len() {
                                list_mut[idx_usize] = new_val.clone();
                                Ok(None)
                            } else {
                                Err(format!(
                                    "Índice {} fuera de rango (longitud: {})",
                                    idx_usize,
                                    list_mut.len()
                                ))
                            }
                        } else {
                            Err("El índice de lista debe ser un número".to_string())
                        }
                    }
                    Value::Diccionario(dict) => {
                        if let Value::Texto(clave) = idx_val {
                            dict.borrow_mut().insert(clave, new_val.clone());
                            Ok(None)
                        } else {
                            Err("La clave de diccionario debe ser texto".to_string())
                        }
                    }
                    _ => {
                        Err("Solo se puede asignar a índices de listas o diccionarios".to_string())
                    }
                }
            }
            Sentencia::AsignacionAtributo {
                objeto,
                atributo,
                valor,
            } => {
                let obj_val = self.evaluar_expresion(objeto).await?;
                let new_val = self.evaluar_expresion(valor).await?;

                if let Value::Instancia { atributos, .. } = obj_val {
                    atributos
                        .borrow_mut()
                        .insert(atributo.clone(), new_val.clone());
                    Ok(None)
                } else {
                    Err(format!(
                        "Solo se pueden asignar atributos a instancias de clase, no a {:?}",
                        obj_val
                    ))
                }
            }
            Sentencia::Expresion(expr) => {
                self.evaluar_expresion(expr).await?;
                Ok(None)
            }
            Sentencia::Imprimir(expr) => {
                let val = self.evaluar_expresion(expr).await?;
                println!("{}", val.a_texto());
                Ok(None)
            }
            Sentencia::Si {
                condicion,
                si_bloque,
                sino_bloque,
            } => {
                let cond = self.evaluar_expresion(condicion).await?;
                if cond.a_booleano() {
                    for sent in si_bloque {
                        if let Some(val) = self.ejecutar_sentencia(sent).await? {
                            return Ok(Some(val));
                        }
                    }
                } else if let Some(sino) = sino_bloque {
                    for sent in sino {
                        if let Some(val) = self.ejecutar_sentencia(sent).await? {
                            return Ok(Some(val));
                        }
                    }
                }
                Ok(None)
            }
            Sentencia::Mientras { condicion, bloque } => {
                loop {
                    let cond = self.evaluar_expresion(condicion).await?;
                    if !cond.a_booleano() {
                        break;
                    }
                    for sent in bloque {
                        if let Some(val) = self.ejecutar_sentencia(sent).await? {
                            return Ok(Some(val));
                        }
                        if self.romper_actual {
                            self.romper_actual = false;
                            return Ok(None);
                        }
                        if self.continuar_actual {
                            self.continuar_actual = false;
                            break; // Rompe el bucle interno para pasar a la siguiente iteración del 'mientras'
                        }
                    }
                    if self.romper_actual {
                        // Si se rompió el bucle interno, también romper el externo
                        self.romper_actual = false;
                        break;
                    }
                }
                Ok(None)
            }
            Sentencia::Para {
                variable,
                iterador,
                bloque,
            } => {
                let iter_val = self.evaluar_expresion(iterador).await?;
                match iter_val {
                    Value::Lista(lista) => {
                        for item in lista.borrow().iter() {
                            if let Some(scope) = self.variables.last_mut() {
                                scope.borrow_mut().insert(variable.clone(), item.clone());
                            }
                            for sent in bloque {
                                if let Some(val) = self.ejecutar_sentencia(sent).await? {
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
                        }
                    }
                    Value::Diccionario(dict) => {
                        for (clave, _valor) in dict.borrow().iter() {
                            if let Some(scope) = self.variables.last_mut() {
                                scope
                                    .borrow_mut()
                                    .insert(variable.clone(), Value::Texto(clave.clone()));
                            }
                            for sent in bloque {
                                if let Some(val) = self.ejecutar_sentencia(sent).await? {
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
                        }
                    }
                    Value::Conjunto(set) => {
                        for item in set.borrow().iter() {
                            if let Some(scope) = self.variables.last_mut() {
                                scope.borrow_mut().insert(variable.clone(), item.clone());
                            }
                            for sent in bloque {
                                if let Some(val) = self.ejecutar_sentencia(sent).await? {
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
                        }
                    }
                    _ => {
                        return Err(
                            "Solo se puede iterar sobre listas, diccionarios o conjuntos"
                                .to_string(),
                        )
                    }
                }
                Ok(None)
            }
            Sentencia::ParaRango {
                variable,
                inicio,
                fin,
                bloque,
            } => {
                let inicio_val = self.evaluar_expresion(inicio).await?;
                let fin_val = self.evaluar_expresion(fin).await?;

                let start = inicio_val.a_numero() as i64;
                let end = fin_val.a_numero() as i64;

                for i in start..end {
                    if let Some(scope) = self.variables.last_mut() {
                        scope
                            .borrow_mut()
                            .insert(variable.clone(), Value::Numero(i as f64));
                    }
                    for sent in bloque {
                        if let Some(val) = self.ejecutar_sentencia(sent).await? {
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
                }
                Ok(None)
            }
            Sentencia::Funcion {
                nombre,
                parametros,
                retorno_tipo,
                bloque,
                es_asincrona,
            } => {
                self.funciones.insert(
                    nombre.clone(),
                    (
                        parametros.clone(),
                        retorno_tipo.clone(),
                        bloque.clone(),
                        *es_asincrona,
                    ),
                );
                Ok(None)
            }
            Sentencia::Clase {
                nombre,
                padre,
                atributos,
                metodos,
            } => {
                self.clases.insert(
                    nombre.clone(),
                    (padre.clone(), atributos.clone(), metodos.clone()),
                );
                Ok(None)
            }
            Sentencia::Importar { ruta, alias } => {
                // 1. Resolver nombre del módulo
                let path = Path::new(ruta);
                // let nombre_modulo = alias.clone().unwrap_or_else(|| {
                //     path.file_stem()
                //         .and_then(|s| s.to_str())
                //         .unwrap_or("modulo")
                //         .to_string()
                // });

                // 2. Leer archivo
                let contenido = fs::read_to_string(ruta)
                    .map_err(|e| format!("Error al importar '{}': {}", ruta, e))?;

                // 3. Parsear
                let mut lexer = Lexer::nuevo(&contenido);
                let tokens = lexer.tokenizar();
                let mut parser = Parser::nuevo(tokens);
                let programa = parser.parsear()?;

                // 4. Ejecutar en nuevo intérprete
                let mut modulo_interprete = Interprete::nuevo();
                modulo_interprete.ejecutar(programa).await?;

                // 5. Extraer variables globales y funciones
                let mut exportaciones = HashMap::new();

                // Variables
                if let Some(global_scope) = modulo_interprete.variables.first() {
                    for (k, v) in global_scope.borrow().iter() {
                        exportaciones.insert(k.clone(), v.clone());
                    }
                }

                // Funciones
                // let global_scope_ref = modulo_interprete.variables.first().unwrap().clone();
                for (nombre_func, (params, _retorno_tipo, body, es_asincrona)) in
                    modulo_interprete.funciones.iter()
                {
                    let val_func = Value::Funcion(
                        params.iter().map(|(n, _)| n.clone()).collect(),
                        body.clone(),
                        Rc::new(RefCell::new(HashMap::new())), // Closure vacío por ahora para globales
                        *es_asincrona,
                    );
                    exportaciones.insert(nombre_func.clone(), val_func);
                }

                let modulo_obj = Value::Diccionario(Rc::new(RefCell::new(exportaciones)));

                if let Some(scope) = self.variables.last_mut() {
                    scope.borrow_mut().insert(
                        alias.clone().unwrap_or_else(|| {
                            path.file_stem().unwrap().to_str().unwrap().to_string()
                        }),
                        modulo_obj,
                    );
                }
                Ok(None)
            }
            Sentencia::Intentar {
                bloque_intentar,
                variable_error,
                bloque_capturar,
            } => {
                let mut error_ocurrido = None;

                // Ejecutar bloque intentar
                for sent in bloque_intentar {
                    match self.ejecutar_sentencia(sent).await {
                        Ok(_) => {
                            if self.retorno_actual.is_some() {
                                return Ok(None);
                            }
                        }
                        Err(msg) => {
                            error_ocurrido = Some(msg);
                            break;
                        }
                    }
                }

                // Si hubo error, ejecutar bloque capturar
                if let Some(msg) = error_ocurrido {
                    let mut nuevo_scope = HashMap::new();
                    nuevo_scope.insert(variable_error.clone(), Value::Texto(msg));

                    self.variables.push(Rc::new(RefCell::new(nuevo_scope)));

                    for sent in bloque_capturar {
                        self.ejecutar_sentencia(sent).await?;
                        if self.retorno_actual.is_some() {
                            break;
                        }
                    }

                    self.variables.pop();
                }

                Ok(None)
            }
            Sentencia::Segun {
                expresion,
                casos,
                defecto,
            } => {
                let valor_evaluado = self.evaluar_expresion(expresion).await?;
                let mut caso_encontrado = false;

                for (caso_expr, bloque) in casos {
                    let valor_caso = self.evaluar_expresion(caso_expr).await?;
                    if valor_evaluado == valor_caso {
                        for sent in bloque {
                            self.ejecutar_sentencia(sent).await?;
                        }
                        caso_encontrado = true;
                        break;
                    }
                }

                if !caso_encontrado {
                    if let Some(bloque_defecto) = defecto {
                        for sent in bloque_defecto {
                            self.ejecutar_sentencia(sent).await?;
                        }
                    }
                }
                Ok(None)
            }
            Sentencia::Retorno(expr) => {
                let val = if let Some(e) = expr {
                    self.evaluar_expresion(e).await?
                } else {
                    Value::Nulo
                };
                // println!("DEBUG: Retornando: {:?}", val);
                return Ok(Some(val));
            }
            Sentencia::Romper => {
                self.romper_actual = true;
                Ok(None)
            }
            Sentencia::Continuar => {
                self.continuar_actual = true;
                Ok(None)
            }
        }
    }

    #[async_recursion(?Send)]
    pub async fn evaluar_expresion(&mut self, expresion: &Expresion) -> Result<Value, String> {
        match expresion {
            Expresion::Numero(n) => Ok(Value::Numero(*n)),
            Expresion::Texto(s) => Ok(Value::Texto(s.clone())),
            Expresion::Logico(b) => Ok(Value::Logico(*b)),
            Expresion::Nulo => Ok(Value::Nulo),
            Expresion::Identificador(nombre) => self.obtener_variable(nombre),
            Expresion::Lista(elementos) => {
                let mut lista = Vec::new();
                for elem in elementos {
                    lista.push(self.evaluar_expresion(elem).await?);
                }
                Ok(Value::Lista(Rc::new(RefCell::new(lista))))
            }
            Expresion::Interpolacion(partes) => {
                let mut resultado = String::new();
                for parte in partes {
                    let val = self.evaluar_expresion(parte).await?;
                    resultado.push_str(&val.a_texto());
                }
                Ok(Value::Texto(resultado))
            }
            Expresion::Diccionario(pares) => {
                let mut dict = HashMap::new();
                for (clave, expr) in pares {
                    let valor = self.evaluar_expresion(expr).await?;
                    dict.insert(clave.clone(), valor);
                }
                Ok(Value::Diccionario(Rc::new(RefCell::new(dict))))
            }
            Expresion::FuncionAnonima {
                parametros,
                bloque,
                es_asincrona,
            } => Ok(Value::Funcion(
                parametros.clone(),
                bloque.clone(),
                self.variables.last().unwrap().clone(),
                *es_asincrona,
            )),
            Expresion::Esperar(expr) => {
                let val = self.evaluar_expresion(expr).await?;
                if let Value::Promesa(fut) = val {
                    fut.await
                } else {
                    Ok(val)
                }
            }
            Expresion::UnOp { op, der } => {
                let val = self.evaluar_expresion(der).await?;
                match op.as_str() {
                    "no" => Ok(Value::Logico(!val.a_logico())),
                    "-" => Ok(Value::Numero(-val.a_numero())),
                    _ => Err(format!("Operador unario desconocido: {}", op)),
                }
            }
            Expresion::BinOp { izq, op, der } => self.evaluar_binop(izq, op, der).await,
            Expresion::Llamada { nombre, args } => self.ejecutar_llamada(nombre, args).await,
            Expresion::MetodoLlamada {
                objeto,
                metodo,
                args,
            } => {
                let obj = self.evaluar_expresion(objeto).await?;
                self.ejecutar_metodo(&obj, metodo, args).await
            }
            Expresion::AccesoAtributo { objeto, atributo } => {
                let obj = self.evaluar_expresion(objeto).await?;
                self.acceder_atributo(&obj, atributo)
            }
            Expresion::AsignacionAtributo {
                objeto,
                atributo,
                valor,
            } => {
                let obj = self.evaluar_expresion(objeto).await?;
                let val = self.evaluar_expresion(valor).await?;
                self.asignar_atributo(&obj, atributo, val)?;
                Ok(Value::Nulo)
            }
            Expresion::Instancia { clase, args } => self.crear_instancia(clase, args).await,
            Expresion::AccesoIndice { objeto, indice } => {
                let obj = self.evaluar_expresion(objeto).await?;
                let idx = self.evaluar_expresion(indice).await?;

                match obj {
                    Value::Lista(lista) => {
                        if let Value::Numero(n) = idx {
                            let i = n as usize;
                            let l = lista.borrow();
                            if i < l.len() {
                                Ok(l[i].clone())
                            } else {
                                Err(format!("Índice fuera de rango: {}", i))
                            }
                        } else {
                            Err("El índice de lista debe ser un número".to_string())
                        }
                    }
                    Value::Diccionario(dict) => {
                        if let Value::Texto(clave) = idx {
                            if let Some(val) = dict.borrow().get(&clave) {
                                Ok(val.clone())
                            } else {
                                Err(format!("Clave '{}' no encontrada en diccionario", clave))
                            }
                        } else {
                            Err("La clave de diccionario debe ser texto".to_string())
                        }
                    }
                    _ => Err("Solo se pueden indexar listas y diccionarios".to_string()),
                }
            }
        }
    }

    async fn evaluar_binop(
        &mut self,
        izq: &Expresion,
        op: &str,
        der: &Expresion,
    ) -> Result<Value, String> {
        let izq_val = self.evaluar_expresion(izq).await?;
        let der_val = self.evaluar_expresion(der).await?;

        match op {
            "+" => match (&izq_val, &der_val) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Numero(a + b)),
                (Value::Texto(a), Value::Texto(b)) => Ok(Value::Texto(format!("{}{}", a, b))),
                (Value::Texto(a), b) => Ok(Value::Texto(format!("{}{}", a, b.a_texto()))),
                (a, Value::Texto(b)) => Ok(Value::Texto(format!("{}{}", a.a_texto(), b))),
                _ => Err("Operación + no válida".to_string()),
            },
            "-" => match (&izq_val, &der_val) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Numero(a - b)),
                _ => Err("Operación - no válida".to_string()),
            },
            "*" => Ok(Value::Numero(izq_val.a_numero() * der_val.a_numero())),
            "/" => {
                let den = der_val.a_numero();
                if den == 0.0 {
                    Err("División por cero".to_string())
                } else {
                    Ok(Value::Numero(izq_val.a_numero() / den))
                }
            }
            "%" => {
                let den = der_val.a_numero();
                if den == 0.0 {
                    Err("División por cero (módulo)".to_string())
                } else {
                    Ok(Value::Numero(izq_val.a_numero() % den))
                }
            }
            "//" => {
                let den = der_val.a_numero();
                if den == 0.0 {
                    Err("División por cero (división entera)".to_string())
                } else {
                    Ok(Value::Numero((izq_val.a_numero() / den).floor()))
                }
            }
            "^" => Ok(Value::Numero(izq_val.a_numero().powf(der_val.a_numero()))),
            "y" => Ok(Value::Logico(izq_val.a_logico() && der_val.a_logico())),
            "o" => Ok(Value::Logico(izq_val.a_logico() || der_val.a_logico())),
            "==" => Ok(Value::Logico(izq_val == der_val)),
            "!=" => Ok(Value::Logico(izq_val != der_val)),
            ">" => match (&izq_val, &der_val) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Logico(a > b)),
                (Value::Texto(a), Value::Texto(b)) => Ok(Value::Logico(a > b)),
                _ => Err("Operación > no válida".to_string()),
            },
            "<" => match (&izq_val, &der_val) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Logico(a < b)),
                (Value::Texto(a), Value::Texto(b)) => Ok(Value::Logico(a < b)),
                _ => Err("Operación < no válida".to_string()),
            },
            ">=" => match (&izq_val, &der_val) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Logico(a >= b)),
                (Value::Texto(a), Value::Texto(b)) => Ok(Value::Logico(a >= b)),
                _ => Err("Operación >= no válida".to_string()),
            },
            "<=" => match (&izq_val, &der_val) {
                (Value::Numero(a), Value::Numero(b)) => Ok(Value::Logico(a <= b)),
                (Value::Texto(a), Value::Texto(b)) => Ok(Value::Logico(a <= b)),
                _ => Err("Operación <= no válida".to_string()),
            },
            _ => Err(format!("Operador {} no reconocido", op)),
        }
    }

    fn obtener_variable(&self, nombre: &str) -> Result<Value, String> {
        for scope in self.variables.iter().rev() {
            if let Some(val) = scope.borrow().get(nombre) {
                return Ok(val.clone());
            }
        }
        Err(format!("Error: Variable '{}' no definida", nombre))
    }

    fn obtener_variable_val(&self, nombre: &str) -> Option<Value> {
        for scope in self.variables.iter().rev() {
            if let Some(val) = scope.borrow().get(nombre) {
                return Some(val.clone());
            }
        }
        None
    }

    #[async_recursion(?Send)]
    async fn ejecutar_llamada(
        &mut self,
        nombre: &str,
        args: &[Expresion],
    ) -> Result<Value, String> {
        // Primero, intenta como clase (instancia)
        if self.clases.contains_key(nombre) {
            return self.crear_instancia(nombre, args).await;
        }

        // Luego, intenta como función nativa
        // Luego, intenta como función nativa o closure
        if let Some(val) = self.obtener_variable_val(nombre) {
            return match val {
                Value::FuncionNativa(f) => {
                    let mut args_vals = Vec::new();
                    for arg in args {
                        args_vals.push(self.evaluar_expresion(arg).await?);
                    }
                    return (f.0)(&args_vals);
                }
                Value::Funcion(parametros, bloque, closure, es_asincrona) => {
                    let mut args_vals = Vec::new();
                    for arg in args {
                        args_vals.push(self.evaluar_expresion(arg).await?);
                    }

                    if es_asincrona {
                        let mut interpreter_clone = self.clone();
                        let parametros = parametros.clone();
                        let bloque = bloque.clone();
                        let closure = closure.clone();
                        let args_vals = args_vals.clone();

                        let future = async move {
                            let mut nuevo_scope = closure.borrow().clone();
                            for (i, param_name) in parametros.iter().enumerate() {
                                let arg_val = if i < args_vals.len() {
                                    args_vals[i].clone()
                                } else {
                                    Value::Nulo
                                };
                                nuevo_scope.insert(param_name.clone(), arg_val);
                            }
                            interpreter_clone
                                .variables
                                .push(Rc::new(RefCell::new(nuevo_scope)));

                            for sent in bloque {
                                if let Some(val) =
                                    interpreter_clone.ejecutar_sentencia(&sent).await?
                                {
                                    interpreter_clone.variables.pop();
                                    return Ok(val);
                                }
                            }
                            interpreter_clone.variables.pop();
                            Ok(Value::Nulo)
                        };

                        return Ok(Value::Promesa(Box::pin(future).boxed_local().shared()));
                    }

                    let mut nuevo_scope = closure.borrow().clone();

                    for (i, param_name) in parametros.iter().enumerate() {
                        let arg_val = if i < args_vals.len() {
                            args_vals[i].clone()
                        } else {
                            Value::Nulo
                        };
                        nuevo_scope.insert(param_name.clone(), arg_val);
                    }

                    self.variables.push(Rc::new(RefCell::new(nuevo_scope)));

                    for sent in bloque {
                        if let Some(val) = self.ejecutar_sentencia(&sent.clone()).await? {
                            self.variables.pop();
                            return Ok(val);
                        }
                    }

                    self.variables.pop();
                    Ok(Value::Nulo)
                }
                _ => Err(format!("'{}' no es una función", nombre)),
            };
        }

        // Luego, intenta como función definida globalmente
        if let Some((parametros, _retorno_tipo, bloque, es_asincrona)) =
            self.funciones.get(nombre).cloned()
        {
            let mut args_vals = Vec::new();
            for arg in args {
                args_vals.push(self.evaluar_expresion(arg).await?);
            }

            if es_asincrona {
                let mut interpreter_clone = self.clone();
                let parametros = parametros.clone();
                let bloque = bloque.clone();
                let args_vals = args_vals.clone();

                let future = async move {
                    let mut nuevo_scope = HashMap::new();
                    for (i, (param_name, _)) in parametros.iter().enumerate() {
                        let arg_val = if i < args_vals.len() {
                            args_vals[i].clone()
                        } else {
                            Value::Nulo
                        };
                        nuevo_scope.insert(param_name.clone(), arg_val);
                    }
                    interpreter_clone
                        .variables
                        .push(Rc::new(RefCell::new(nuevo_scope)));

                    for sent in &bloque {
                        if let Some(val) = interpreter_clone.ejecutar_sentencia(sent).await? {
                            interpreter_clone.variables.pop();
                            return Ok(val);
                        }
                    }
                    interpreter_clone.variables.pop();
                    Ok(interpreter_clone
                        .retorno_actual
                        .take()
                        .unwrap_or(Value::Nulo))
                };
                return Ok(Value::Promesa(Box::pin(future).boxed_local().shared()));
            }

            let mut nuevo_scope = HashMap::new();

            for (i, (param_name, param_type)) in parametros.iter().enumerate() {
                let arg_val = if i < args_vals.len() {
                    args_vals[i].clone()
                } else {
                    Value::Nulo
                };

                if let Some(tipo) = param_type {
                    if !self.verificar_tipo(&arg_val, tipo) {
                        return Err(format!(
                            "Error de Tipo: Argumento '{}' espera {}, se recibió {:?}",
                            param_name, tipo, arg_val
                        ));
                    }
                }

                nuevo_scope.insert(param_name.clone(), arg_val);
            }

            self.variables.push(Rc::new(RefCell::new(nuevo_scope)));

            for sent in &bloque {
                if let Some(val) = self.ejecutar_sentencia(sent).await? {
                    self.variables.pop();
                    return Ok(val);
                }
            }

            self.variables.pop();
            Ok(self.retorno_actual.take().unwrap_or(Value::Nulo))
        } else {
            Err(format!("Error: Función '{}' no definida", nombre))
        }
    }

    async fn crear_instancia(
        &mut self,
        clase_nombre: &str,
        args: &[Expresion],
    ) -> Result<Value, String> {
        // 1. Buscar la clase
        if let Some(val_clase) = self.obtener_variable_val(clase_nombre) {
            if let Value::Clase(_, _) = val_clase {
                // ... (lógica anterior que usaba Value::Clase directamente)
                // Pero ahora usamos self.clases
                return Err("Lógica de instanciación antigua detectada".to_string());
            }
        }

        // Nueva lógica usando self.clases
        if let Some((_, scope_clase, metodos_all)) = self.clases.get(clase_nombre).cloned() {
            // 2. Crear el objeto con los atributos iniciales (copia del scope de clase)
            let mut objeto = HashMap::new();
            // scope_clase es Vec<(String, Option<String>)> (atributos definidos en la clase)
            for (attr_name, _) in scope_clase.iter() {
                objeto.insert(attr_name.clone(), Value::Nulo);
            }

            let instancia = Value::Instancia {
                clase: clase_nombre.to_string(),
                atributos: Rc::new(RefCell::new(objeto)),
            };

            // Buscar y ejecutar constructor (__init__)
            for (metodo_nombre, parametros, bloque) in metodos_all {
                if metodo_nombre == "nuevo" {
                    // Ejecutar constructor (similar a ejecutar_metodo)
                    let mut valores_args = Vec::new();
                    for arg in args {
                        valores_args.push(self.evaluar_expresion(arg).await?);
                    }

                    if valores_args.len() != parametros.len() {
                        return Err(format!(
                            "Constructor espera {} argumentos, se recibieron {}",
                            parametros.len(),
                            valores_args.len()
                        ));
                    }

                    let mut scope_metodo = HashMap::new();
                    scope_metodo.insert("yo".to_string(), instancia.clone());

                    for (i, (param, _)) in parametros.iter().enumerate() {
                        scope_metodo.insert(param.clone(), valores_args[i].clone());
                    }

                    self.variables.push(Rc::new(RefCell::new(scope_metodo)));
                    let programa = crate::ast::Programa {
                        sentencias: bloque.clone(),
                    };
                    self.ejecutar(programa).await?;
                    self.variables.pop();

                    return Ok(instancia);
                }
            }

            Ok(instancia)
        } else {
            Err(format!("Clase '{}' no definida", clase_nombre))
        }
    }

    fn registrar_modulo_db(&mut self) {
        let mut metodos_db = HashMap::new();

        metodos_db.insert(
            "conectar".to_string(),
            Value::FuncionNativa(crate::types::NativeFn(Rc::new(|args: &[Value]| {
                if args.len() != 1 {
                    return Err("db.conectar espera 1 argumento (url)".to_string());
                }
                if let Value::Texto(url) = &args[0] {
                    match postgres::Client::connect(url.as_str(), postgres::NoTls) {
                        Ok(client) => Ok(Value::BaseDeDatos(crate::types::DbClient(Rc::new(
                            RefCell::new(client),
                        )))),
                        Err(e) => Err(format!("Error de conexión: {}", e)),
                    }
                } else {
                    Err("db.conectar espera texto".to_string())
                }
            }))),
        );

        let db_modulo = Value::Diccionario(Rc::new(RefCell::new(metodos_db)));
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("db".to_string(), db_modulo);
        }
    }

    fn registrar_modulo_util(&mut self) {
        let mut metodos_util = HashMap::new();

        metodos_util.insert(
            "dormir".to_string(),
            Value::FuncionNativa(crate::types::NativeFn(Rc::new(|args: &[Value]| {
                if args.len() != 1 {
                    return Err("util.dormir espera 1 argumento (milisegundos)".to_string());
                }
                let ms = args[0].a_numero() as u64;

                let future = async move {
                    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
                    Ok(Value::Nulo)
                };

                Ok(Value::Promesa(Box::pin(future).boxed_local().shared()))
            }))),
        );

        let util_modulo = Value::Diccionario(Rc::new(RefCell::new(metodos_util)));
        if let Some(scope) = self.variables.first_mut() {
            scope.borrow_mut().insert("util".to_string(), util_modulo);
        }
    }

    fn evaluar_metodo(
        &mut self,
        objeto: Value,
        metodo: String,
        args: Vec<Value>,
    ) -> Result<Value, String> {
        match objeto {
            Value::BaseDeDatos(db_client) => {
                let mut client = db_client.0.borrow_mut();
                match metodo.as_str() {
                    "consulta" => {
                        if args.len() != 1 {
                            return Err("consulta espera 1 argumento (sql)".to_string());
                        }
                        if let Value::Texto(sql) = &args[0] {
                            match client.query(sql.as_str(), &[]) {
                                Ok(filas) => {
                                    let mut resultados = Vec::new();
                                    for fila in filas {
                                        let mut dic = HashMap::new();
                                        for (i, columna) in fila.columns().iter().enumerate() {
                                            let nombre = columna.name().to_string();
                                            // Mapeo básico de tipos
                                            let valor = if let Ok(v) = fila.try_get::<_, String>(i)
                                            {
                                                Value::Texto(v)
                                            } else if let Ok(v) = fila.try_get::<_, i32>(i) {
                                                Value::Numero(v as f64)
                                            } else if let Ok(v) = fila.try_get::<_, bool>(i) {
                                                Value::Logico(v)
                                            } else {
                                                Value::Nulo
                                            };
                                            dic.insert(nombre, valor);
                                        }
                                        resultados
                                            .push(Value::Diccionario(Rc::new(RefCell::new(dic))));
                                    }
                                    Ok(Value::Lista(Rc::new(RefCell::new(resultados))))
                                }
                                Err(e) => Err(format!("Error en consulta: {}", e)),
                            }
                        } else {
                            Err("consulta espera texto SQL".to_string())
                        }
                    }
                    "ejecutar" => {
                        if args.len() != 1 {
                            return Err("ejecutar espera 1 argumento (sql)".to_string());
                        }
                        if let Value::Texto(sql) = &args[0] {
                            match client.execute(sql.as_str(), &[]) {
                                Ok(filas_afectadas) => Ok(Value::Numero(filas_afectadas as f64)),
                                Err(e) => Err(format!("Error en ejecución: {}", e)),
                            }
                        } else {
                            Err("ejecutar espera texto SQL".to_string())
                        }
                    }
                    "cerrar" => Ok(Value::Nulo),
                    _ => Err(format!("Método '{}' no definido para BaseDeDatos", metodo)),
                }
            }
            Value::Lista(lista_rc) => match metodo.as_str() {
                "agregar" | "eliminar" | "insertar" | "ordenar" | "invertir" | "limpiar" => {
                    let mut lista = lista_rc.borrow_mut();
                    match metodo.as_str() {
                        "agregar" => {
                            if args.len() != 1 {
                                return Err("agregar espera 1 argumento".to_string());
                            }
                            lista.push(args[0].clone());
                            Ok(Value::Nulo)
                        }
                        "eliminar" => {
                            if args.len() != 1 {
                                return Err("eliminar espera 1 argumento (indice)".to_string());
                            }
                            let indice = args[0].a_numero() as usize;
                            if indice < lista.len() {
                                Ok(lista.remove(indice))
                            } else {
                                Err(format!("Índice fuera de rango: {}", indice))
                            }
                        }
                        "insertar" => {
                            if args.len() != 2 {
                                return Err(
                                    "insertar espera 2 argumentos (indice, valor)".to_string()
                                );
                            }
                            let indice = args[0].a_numero() as usize;
                            if indice <= lista.len() {
                                lista.insert(indice, args[1].clone());
                                Ok(Value::Nulo)
                            } else {
                                Err(format!("Índice fuera de rango: {}", indice))
                            }
                        }
                        "ordenar" => {
                            lista.sort_by(|a, b| match (a, b) {
                                (Value::Numero(n1), Value::Numero(n2)) => {
                                    n1.partial_cmp(n2).unwrap_or(std::cmp::Ordering::Equal)
                                }
                                (Value::Texto(s1), Value::Texto(s2)) => s1.cmp(s2),
                                _ => std::cmp::Ordering::Equal,
                            });
                            Ok(Value::Nulo)
                        }
                        "invertir" => {
                            lista.reverse();
                            Ok(Value::Nulo)
                        }
                        "limpiar" => {
                            lista.clear();
                            Ok(Value::Nulo)
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    let lista = lista_rc.borrow();
                    match metodo.as_str() {
                        "longitud" => Ok(Value::Numero(lista.len() as f64)),
                        "suma" => {
                            let sum: f64 = lista
                                .iter()
                                .filter_map(|v| {
                                    if let Value::Numero(n) = v {
                                        Some(n)
                                    } else {
                                        None
                                    }
                                })
                                .sum();
                            Ok(Value::Numero(sum))
                        }
                        "minimo" => lista
                            .iter()
                            .filter_map(|v| {
                                if let Value::Numero(n) = v {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                            .map(Value::Numero)
                            .ok_or("Lista vacía o sin números".to_string()),
                        "maximo" => lista
                            .iter()
                            .filter_map(|v| {
                                if let Value::Numero(n) = v {
                                    Some(*n)
                                } else {
                                    None
                                }
                            })
                            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                            .map(Value::Numero)
                            .ok_or("Lista vacía o sin números".to_string()),
                        "contiene" => {
                            if args.len() != 1 {
                                return Err("contiene espera 1 argumento".to_string());
                            }
                            let buscado = &args[0];
                            let encontrado = lista.iter().any(|v| v == buscado);
                            Ok(Value::Logico(encontrado))
                        }
                        "copiar" => Ok(Value::Lista(Rc::new(RefCell::new(lista.clone())))),
                        "unir" => {
                            if args.len() != 1 {
                                return Err("unir espera 1 argumento (separador)".to_string());
                            }
                            let sep = args[0].a_texto();
                            let strs: Vec<String> = lista.iter().map(|v| v.a_texto()).collect();
                            Ok(Value::Texto(strs.join(&sep)))
                        }
                        "sublista" => {
                            if args.len() != 2 {
                                return Err(
                                    "sublista espera 2 argumentos (inicio, fin)".to_string()
                                );
                            }
                            let inicio = args[0].a_numero() as usize;
                            let fin = args[1].a_numero() as usize;
                            if inicio <= fin && fin <= lista.len() {
                                let sub = lista[inicio..fin].to_vec();
                                Ok(Value::Lista(Rc::new(RefCell::new(sub))))
                            } else {
                                Err("Índices fuera de rango".to_string())
                            }
                        }
                        "a_texto" => {
                            let strs: Vec<String> = lista.iter().map(|v| v.a_texto()).collect();
                            Ok(Value::Texto(format!("[{}]", strs.join(", "))))
                        }
                        _ => Err(format!("Método '{}' no definido para Lista", metodo)),
                    }
                }
            },
            Value::Numero(n) => match metodo.as_str() {
                "redondear" => {
                    if args.len() == 0 {
                        Ok(Value::Numero(n.round()))
                    } else if args.len() == 1 {
                        let decimales = args[0].a_numero();
                        let factor = 10f64.powf(decimales);
                        Ok(Value::Numero((n * factor).round() / factor))
                    } else {
                        Err("redondear espera 0 o 1 argumento".to_string())
                    }
                }
                "piso" => Ok(Value::Numero(n.floor())),
                "techo" => Ok(Value::Numero(n.ceil())),
                "abs" => Ok(Value::Numero(n.abs())),
                "a_texto" => Ok(Value::Texto(n.to_string())),
                _ => Err(format!("Método '{}' no definido para Numero", metodo)),
            },
            Value::Texto(s) => {
                match metodo.as_str() {
                    "a_numero" => match s.parse::<f64>() {
                        Ok(n) => Ok(Value::Numero(n)),
                        Err(_) => Ok(Value::Nulo),
                    },
                    "longitud" => Ok(Value::Numero(s.chars().count() as f64)),
                    "mayusculas" => Ok(Value::Texto(s.to_uppercase())),
                    "minusculas" => Ok(Value::Texto(s.to_lowercase())),
                    "contiene" => {
                        if args.len() != 1 {
                            return Err("contiene espera 1 argumento".to_string());
                        }
                        let sub = args[0].a_texto();
                        Ok(Value::Logico(s.contains(&sub)))
                    }
                    "reemplazar" => {
                        if args.len() != 2 {
                            return Err("reemplazar espera 2 argumentos (viejo, nuevo)".to_string());
                        }
                        let viejo = args[0].a_texto();
                        let nuevo = args[1].a_texto();
                        Ok(Value::Texto(s.replace(&viejo, &nuevo)))
                    }
                    "dividir" => {
                        if args.len() != 1 {
                            return Err("dividir espera 1 argumento (separador)".to_string());
                        }
                        let sep = args[0].a_texto();
                        let partes: Vec<Value> =
                            s.split(&sep).map(|p| Value::Texto(p.to_string())).collect();
                        Ok(Value::Lista(Rc::new(RefCell::new(partes))))
                    }
                    "recortar" => Ok(Value::Texto(s.trim().to_string())),
                    "caracter_en" => {
                        if args.len() != 1 {
                            return Err("caracter_en espera 1 argumento (indice)".to_string());
                        }
                        let idx = args[0].a_numero() as usize;
                        if let Some(c) = s.chars().nth(idx) {
                            Ok(Value::Texto(c.to_string()))
                        } else {
                            Ok(Value::Texto("".to_string())) // Retornar vacio si fuera de rango para evitar panic
                        }
                    }
                    "subcadena" => {
                        if args.len() != 2 {
                            return Err("subcadena espera 2 argumentos (inicio, fin)".to_string());
                        }
                        let inicio = args[0].a_numero() as usize;
                        let fin = args[1].a_numero() as usize;
                        let chars: Vec<char> = s.chars().collect();
                        if inicio <= fin && fin <= chars.len() {
                            let sub: String = chars[inicio..fin].iter().collect();
                            Ok(Value::Texto(sub))
                        } else {
                            Ok(Value::Texto("".to_string())) // Tolerante a fallos
                        }
                    }
                    _ => Err(format!("Método '{}' no definido para Texto", metodo)),
                }
            }
            Value::Diccionario(map_rc) => {
                let mut map = map_rc.borrow_mut();
                match metodo.as_str() {
                    "claves" => {
                        let claves: Vec<Value> =
                            map.keys().map(|k| Value::Texto(k.clone())).collect();
                        Ok(Value::Lista(Rc::new(RefCell::new(claves))))
                    }
                    "valores" => {
                        let valores: Vec<Value> = map.values().cloned().collect();
                        Ok(Value::Lista(Rc::new(RefCell::new(valores))))
                    }
                    "insertar" => {
                        if args.len() != 2 {
                            return Err("insertar espera 2 argumentos (clave, valor)".to_string());
                        }
                        let clave = args[0].a_texto();
                        map.insert(clave, args[1].clone());
                        Ok(Value::Nulo)
                    }
                    "longitud" => Ok(Value::Numero(map.len() as f64)),
                    "contiene" => {
                        if args.len() != 1 {
                            return Err("contiene espera 1 argumento (clave)".to_string());
                        }
                        let clave = args[0].a_texto();
                        Ok(Value::Logico(map.contains_key(&clave)))
                    }
                    "obtener" => {
                        if args.len() < 1 {
                            return Err("obtener espera al menos 1 argumento (clave)".to_string());
                        }
                        let clave = args[0].a_texto();
                        if let Some(val) = map.get(&clave) {
                            Ok(val.clone())
                        } else if args.len() > 1 {
                            Ok(args[1].clone())
                        } else {
                            Ok(Value::Nulo)
                        }
                    }
                    "eliminar" => {
                        if args.len() != 1 {
                            return Err("eliminar espera 1 argumento (clave)".to_string());
                        }
                        let clave = args[0].a_texto();
                        Ok(map.remove(&clave).unwrap_or(Value::Nulo))
                    }
                    "limpiar" => {
                        map.clear();
                        Ok(Value::Nulo)
                    }
                    "copiar" => Ok(Value::Diccionario(Rc::new(RefCell::new(map.clone())))),
                    _ => Err(format!("Método '{}' no definido para Diccionario", metodo)),
                }
            }
            Value::Conjunto(set_rc) => {
                let mut set = set_rc.borrow_mut();
                match metodo.as_str() {
                    "agregar" => {
                        if args.len() != 1 {
                            return Err("agregar espera 1 argumento".to_string());
                        }
                        set.insert(args[0].clone());
                        Ok(Value::Nulo)
                    }
                    "eliminar" => {
                        if args.len() != 1 {
                            return Err("eliminar espera 1 argumento".to_string());
                        }
                        Ok(Value::Logico(set.remove(&args[0])))
                    }
                    "contiene" => {
                        if args.len() != 1 {
                            return Err("contiene espera 1 argumento".to_string());
                        }
                        Ok(Value::Logico(set.contains(&args[0])))
                    }
                    "longitud" => Ok(Value::Numero(set.len() as f64)),
                    "a_lista" => {
                        let lista: Vec<Value> = set.iter().cloned().collect();
                        Ok(Value::Lista(Rc::new(RefCell::new(lista))))
                    }
                    "unir" => {
                        if args.len() != 1 {
                            return Err("unir espera 1 argumento (otro conjunto)".to_string());
                        }
                        if let Value::Conjunto(otro_rc) = &args[0] {
                            let otro = otro_rc.borrow();
                            let union: std::collections::HashSet<_> =
                                set.union(&otro).cloned().collect();
                            Ok(Value::Conjunto(Rc::new(RefCell::new(union))))
                        } else {
                            Err("El argumento de unir debe ser un conjunto".to_string())
                        }
                    }
                    "intersectar" => {
                        if args.len() != 1 {
                            return Err(
                                "intersectar espera 1 argumento (otro conjunto)".to_string()
                            );
                        }
                        if let Value::Conjunto(otro_rc) = &args[0] {
                            let otro = otro_rc.borrow();
                            let inter: std::collections::HashSet<_> =
                                set.intersection(&otro).cloned().collect();
                            Ok(Value::Conjunto(Rc::new(RefCell::new(inter))))
                        } else {
                            Err("El argumento de intersectar debe ser un conjunto".to_string())
                        }
                    }
                    "diferencia" => {
                        if args.len() != 1 {
                            return Err("diferencia espera 1 argumento (otro conjunto)".to_string());
                        }
                        if let Value::Conjunto(otro_rc) = &args[0] {
                            let otro = otro_rc.borrow();
                            let diff: std::collections::HashSet<_> =
                                set.difference(&otro).cloned().collect();
                            Ok(Value::Conjunto(Rc::new(RefCell::new(diff))))
                        } else {
                            Err("El argumento de diferencia debe ser un conjunto".to_string())
                        }
                    }
                    _ => Err(format!("Método '{}' no definido para Conjunto", metodo)),
                }
            }
            _ => Err(format!(
                "No se puede llamar al método '{}' en este tipo de objeto",
                metodo
            )),
        }
    }

    #[async_recursion(?Send)]
    async fn ejecutar_metodo(
        &mut self,
        objeto: &Value,
        metodo: &str,
        args: &[Expresion],
    ) -> Result<Value, String> {
        if let Value::Instancia {
            clase: clase_nombre,
            atributos,
        } = objeto
        {
            // 1. Intentar buscar en la definición de la clase
            if let Some((_, _, metodos)) = self.clases.get(clase_nombre).cloned() {
                for (metodo_nombre, parametros, bloque) in metodos {
                    if metodo_nombre == metodo {
                        // Evaluar argumentos
                        let mut valores_args = Vec::new();
                        for arg in args {
                            valores_args.push(self.evaluar_expresion(arg).await?);
                        }

                        if valores_args.len() != parametros.len() {
                            return Err(format!(
                                "Método '{}' espera {} argumentos, se recibieron {}",
                                metodo,
                                parametros.len(),
                                valores_args.len()
                            ));
                        }

                        // Crear scope para el método
                        let mut scope_metodo = HashMap::new();
                        // 'this' apunta a la instancia
                        scope_metodo.insert("yo".to_string(), objeto.clone());

                        for (i, (param, _)) in parametros.iter().enumerate() {
                            scope_metodo.insert(param.clone(), valores_args[i].clone());
                        }

                        self.variables.push(Rc::new(RefCell::new(scope_metodo)));
                        let programa = crate::ast::Programa {
                            sentencias: bloque.clone(),
                        };
                        let resultado = self.ejecutar(programa).await;
                        self.variables.pop();

                        return match resultado {
                            Ok(val_opt) => Ok(val_opt.unwrap_or(Value::Nulo)),
                            Err(e) => Err(format!("{}\n  en {}.{}()", e, clase_nombre, metodo)),
                        };
                    }
                }
            }

            // 2. Intentar buscar en los atributos del objeto (para métodos dinámicos)
            if let Some(val) = atributos.borrow().get(metodo) {
                match val {
                    Value::FuncionNativa(native_fn) => {
                        let mut valores_args = Vec::new();
                        for arg in args {
                            valores_args.push(self.evaluar_expresion(arg).await?);
                        }
                        return (native_fn.0)(&valores_args);
                    }
                    Value::Funcion(..) => {
                        return Err(
                            "Llamada a función en atributo no implementada completamente"
                                .to_string(),
                        );
                    }
                    _ => {}
                }
            }

            // Si no se encuentra en clase ni atributos, continuamos para intentar método nativo
        } else if let Value::Diccionario(map) = objeto {
            // Soporte para módulos (que son diccionarios de funciones)
            if let Some(val) = map.borrow().get(metodo) {
                match val {
                    Value::FuncionNativa(native_fn) => {
                        let mut valores_args = Vec::new();
                        for arg in args {
                            valores_args.push(self.evaluar_expresion(arg).await?);
                        }
                        return (native_fn.0)(&valores_args);
                    }
                    Value::Funcion(..) => {
                        return Err(
                            "Llamada a función definida por usuario en módulo no implementada aún"
                                .to_string(),
                        );
                    }
                    _ => return Err(format!("'{}' no es una función", metodo)),
                }
            }
        }

        // Si no es instancia ni diccionario, intentar métodos nativos (listas, etc) o DB
        let mut valores_args = Vec::new();
        for arg in args {
            valores_args.push(self.evaluar_expresion(arg).await?);
        }
        self.evaluar_metodo(objeto.clone(), metodo.to_string(), valores_args)
    }

    fn verificar_tipo(&self, valor: &Value, tipo_esperado: &str) -> bool {
        match (valor, tipo_esperado) {
            (Value::Numero(_), "Numero") => true,
            (Value::Texto(_), "Texto") => true,
            (Value::Logico(_), "Logico") => true,
            (Value::Nulo, "Nulo") => true,
            (Value::Lista(_), "Lista") => true,
            (Value::Diccionario(_), "Diccionario") => true,
            (_, "Cualquiera") => true,
            _ => false,
        }
    }

    fn acceder_atributo(&self, objeto: &Value, atributo: &str) -> Result<Value, String> {
        // println!("DEBUG: Accediendo atributo '{}' en objeto de tipo {:?}", atributo, objeto);
        if let Value::Instancia {
            clase: _,
            atributos,
        } = objeto
        {
            if let Some(val) = atributos.borrow().get(atributo) {
                Ok(val.clone())
            } else {
                Err(format!("Error: Atributo '{}' no encontrado", atributo))
            }
        } else if let Value::Clase(_, _) = objeto {
            // TODO: Implementar herencia de atributos estáticos si es necesario
            Err(format!(
                "Error: No se puede acceder al atributo '{}' en la clase",
                atributo
            ))
        } else if let Value::Diccionario(map) = objeto {
            if let Some(val) = map.borrow().get(atributo) {
                Ok(val.clone())
            } else {
                Err(format!(
                    "Error: Clave '{}' no encontrada en diccionario",
                    atributo
                ))
            }
        } else {
            Err("Error: No es un objeto o diccionario".to_string())
        }
    }

    fn asignar_atributo(
        &mut self,
        objeto: &Value,
        atributo: &str,
        valor: Value,
    ) -> Result<(), String> {
        if let Value::Instancia {
            clase: _,
            atributos,
        } = objeto
        {
            atributos.borrow_mut().insert(atributo.to_string(), valor);
            Ok(())
        } else if let Value::Diccionario(map) = objeto {
            map.borrow_mut().insert(atributo.to_string(), valor);
            Ok(())
        } else {
            Err("Error: No es un objeto o diccionario".to_string())
        }
    }
}

// Funciones auxiliares para JSON
fn json_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Nulo,
        serde_json::Value::Bool(b) => Value::Logico(*b),
        serde_json::Value::Number(n) => Value::Numero(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Value::Texto(s.clone()),
        serde_json::Value::Array(arr) => {
            let list = arr.iter().map(json_to_value).collect();
            Value::Lista(Rc::new(RefCell::new(list)))
        }
        serde_json::Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), json_to_value(v));
            }
            Value::Diccionario(Rc::new(RefCell::new(map)))
        }
    }
}

fn value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Nulo => serde_json::Value::Null,
        Value::Logico(b) => serde_json::Value::Bool(*b),
        Value::Numero(n) => serde_json::Number::from_f64(*n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::Texto(s) => serde_json::Value::String(s.clone()),
        Value::Lista(list) => {
            let arr = list.borrow().iter().map(value_to_json).collect();
            serde_json::Value::Array(arr)
        }
        Value::Diccionario(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map.borrow().iter() {
                obj.insert(k.clone(), value_to_json(v));
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::String(v.a_texto()),
    }
}
