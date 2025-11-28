#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literales
    Numero(f64),
    Texto(String),
    TextoInterpolado(String),
    Identificador(String),

    // Palabras clave
    Funcion,
    Si,
    Sino,
    Mientras,
    Para,
    En,
    Hasta,
    Clase,
    Imprimir,
    Verdadero,
    Falso,
    Nulo,
    Importar,
    Retornar,
    Intentar,
    Capturar,
    Nuevo,
    Asincrono,
    Esperar,
    Segun,
    Caso,
    Defecto,
    Romper,

    // Operadores
    Mas,
    Menos,
    Por,
    Div,
    Mayor,
    Menor,
    MayorIgual,
    MenorIgual,
    Igual,
    NoIgual,
    Asignacion,
    Punto,
    DosPuntos,
    Coma,
    Dos,
    Flecha,
    Modulo,
    DivEntera,
    Potencia,
    MasIgual,
    MenosIgual,
    Y,
    O,
    No,

    // Delimitadores
    ParAbre,
    ParCierra,
    LlaveAbre,
    LlaveCierra,
    CorcheteAbre,
    CorcheteCierra,

    // Control
    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sentencia {
    Asignacion {
        nombre: String,
        tipo: Option<String>,
        valor: Expresion,
    },
    AsignacionIndice {
        objeto: Expresion,
        indice: Expresion,
        valor: Expresion,
    },
    Expresion(Expresion),
    Si {
        condicion: Expresion,
        si_bloque: Vec<Sentencia>,
        sino_bloque: Option<Vec<Sentencia>>,
    },
    Mientras {
        condicion: Expresion,
        bloque: Vec<Sentencia>,
    },
    Para {
        variable: String,
        iterador: Expresion,
        bloque: Vec<Sentencia>,
    },
    ParaRango {
        variable: String,
        inicio: i64,
        fin: i64,
        bloque: Vec<Sentencia>,
    },
    Funcion {
        nombre: String,
        parametros: Vec<(String, Option<String>)>,
        retorno_tipo: Option<String>,
        bloque: Vec<Sentencia>,
        es_asincrona: bool,
    },
    Clase {
        nombre: String,
        padre: Option<String>,
        atributos: Vec<(String, Option<String>)>,
        metodos: Vec<(String, Vec<(String, Option<String>)>, Vec<Sentencia>)>,
    },
    Retorno(Option<Expresion>),
    Importar {
        ruta: String,
        alias: Option<String>,
    },
    Intentar {
        bloque_intentar: Vec<Sentencia>,
        variable_error: String,
        bloque_capturar: Vec<Sentencia>,
    },
    Imprimir(Expresion),
    Segun {
        expresion: Expresion,
        casos: Vec<(Expresion, Vec<Sentencia>)>,
        defecto: Option<Vec<Sentencia>>,
    },
    Romper,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expresion {
    Numero(f64),
    Texto(String),
    Logico(bool),
    Nulo,
    Identificador(String),
    Lista(Vec<Expresion>),
    Diccionario(Vec<(String, Expresion)>),
    Interpolacion(Vec<Expresion>),
    BinOp {
        izq: Box<Expresion>,
        op: String,
        der: Box<Expresion>,
    },
    Llamada {
        nombre: String,
        args: Vec<Expresion>,
    },
    MetodoLlamada {
        objeto: Box<Expresion>,
        metodo: String,
        args: Vec<Expresion>,
    },
    AccesoAtributo {
        objeto: Box<Expresion>,
        atributo: String,
    },
    AsignacionAtributo {
        objeto: Box<Expresion>,
        atributo: String,
        valor: Box<Expresion>,
    },
    Instancia {
        clase: String,
        args: Vec<Expresion>,
    },
    AccesoIndice {
        objeto: Box<Expresion>,
        indice: Box<Expresion>,
    },
    FuncionAnonima {
        parametros: Vec<String>,
        bloque: Vec<Sentencia>,
        es_asincrona: bool,
    },
    Esperar(Box<Expresion>),
    UnOp {
        op: String,
        der: Box<Expresion>,
    },
}

#[derive(Debug, Clone)]
pub struct Programa {
    pub sentencias: Vec<Sentencia>,
}
