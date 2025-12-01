#![allow(dead_code)]
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literales
    Numero(f64),
    Texto(String),
    TextoInterpolado(String), // f"Hola {nombre}"
    Identificador(String),

    // Palabras clave (Español)
    Si,         // if
    Sino,       // else
    Mientras,   // while
    Para,       // for
    En,         // in
    Romper,     // break
    Continuar,  // continue
    Retornar,   // return
    Funcion,    // def / fn
    Clase,      // class
    Verdadero,  // True
    Falso,      // False
    Nulo,       // None
    Y,          // and
    O,          // or
    No,         // not
    Importar,   // import
    Desde,      // from
    Como,       // as
    Intentar,   // try
    Capturar,   // except / catch
    Finalmente, // finally
    Lanzar,     // raise / throw
    Global,     // global
    NoLocal,    // nonlocal
    Pasar,      // pass
    Eliminar,   // del
    Con,        // with
    Asincrono,  // async
    Esperar,    // await
    Ceder,      // yield
    Segun,      // match
    Caso,       // case
    Defecto,    // default (opcional en match)
    Imprimir,   // print (como sentencia o función nativa)
    Afirmar,    // assert
    Nuevo,      // new (opcional, si queremos instanciación explícita)
    Let,        // let (opcional, para variables locales explícitas)

    // Operadores Aritméticos
    Mas,       // +
    Menos,     // -
    Por,       // *
    Div,       // /
    DivEntera, // //
    Modulo,    // %
    Potencia,  // **

    // Operadores de Asignación Aumentada
    MasIgual,       // +=
    MenosIgual,     // -=
    PorIgual,       // *=
    DivIgual,       // /=
    DivEnteraIgual, // //=
    ModuloIgual,    // %=
    PotenciaIgual,  // **=

    // Operadores de Comparación
    Igual,      // ==
    NoIgual,    // !=
    Mayor,      // >
    Menor,      // <
    MayorIgual, // >=
    MenorIgual, // <=

    // Operadores Bitwise
    Ampersand, // &
    Barra,     // |
    Caret,     // ^
    Tilde,     // ~
    ShiftIzq,  // <<
    ShiftDer,  // >>

    // Símbolos y Delimitadores
    ParAbre,        // (
    ParCierra,      // )
    CorcheteAbre,   // [
    CorcheteCierra, // ]
    LlaveAbre,      // {
    LlaveCierra,    // }
    Coma,           // ,
    Punto,          // .
    DosPuntos,      // :
    PuntoYComa,     // ;
    Flecha,         // ->
    Arroba,         // @ (Decoradores)
    Asignacion,     // =

    // Especiales
    EOF,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperadorUnario {
    Negativo, // -
    Not,      // no
    BitNot,   // ~
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperadorBinario {
    Suma,
    Resta,
    Multiplicacion,
    Division,
    DivisionEntera,
    Modulo,
    Potencia,
    Igual,
    NoIgual,
    Mayor,
    Menor,
    MayorIgual,
    MenorIgual,
    Y,
    O,
    BitAnd,
    BitOr,
    BitXor,
    ShiftIzq,
    ShiftDer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expresion {
    // Abstract Syntax Tree (AST)Datos
    // Valores Básicos
    Literal(Literal),
    Identificador(String),

    // Estructuras de Datos
    Lista(Vec<Expresion>),
    Diccionario(Vec<(Expresion, Expresion)>),
    Conjunto(Vec<Expresion>),

    // Operaciones
    Binaria {
        izq: Box<Expresion>,
        op: OperadorBinario,
        der: Box<Expresion>,
    },
    Unaria {
        op: OperadorUnario,
        exp: Box<Expresion>,
    },

    // Funciones y Métodos
    Llamada {
        func: Box<Expresion>,
        args: Vec<Expresion>,
    },
    AccesoAtributo {
        objeto: Box<Expresion>,
        atributo: String,
    },
    AccesoIndice {
        objeto: Box<Expresion>,
        indice: Box<Expresion>,
    },

    // Avanzados
    Ternaria {
        condicion: Box<Expresion>,
        verdadero: Box<Expresion>,
        falso: Box<Expresion>,
    },
    Lambda {
        params: Vec<String>,
        cuerpo: Box<Expresion>,
    },

    // Asincronía y Generadores
    Await(Box<Expresion>),
    Yield(Option<Box<Expresion>>),

    // Comprehensions (Listas por comprensión)
    // [x * 2 para x en lista si x > 0]
    ComprehensionLista {
        elemento: Box<Expresion>,
        variable: String,
        iterable: Box<Expresion>,
        condicion: Option<Box<Expresion>>,
    },

    // Interpolación de Strings
    Interpolacion(Vec<Expresion>), // Fragmentos de string y expresiones
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Entero(i64),
    Decimal(f64),
    Texto(String),
    Booleano(bool),
    Nulo,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Sentencia {
    // Básicas
    Expresion(Expresion),
    Asignacion {
        objetivo: Expresion, // Puede ser variable, atributo o índice
        valor: Expresion,
        tipo: Option<String>, // Hint de tipo opcional
    },
    AsignacionAumentada {
        objetivo: Expresion,
        op: OperadorBinario,
        valor: Expresion,
    },

    // Control de Flujo
    Si {
        condicion: Expresion,
        entonces: Vec<Sentencia>,
        sino: Option<Vec<Sentencia>>,
    },
    Mientras {
        condicion: Expresion,
        cuerpo: Vec<Sentencia>,
    },
    Para {
        variable: String,
        iterable: Expresion,
        cuerpo: Vec<Sentencia>,
    },
    Segun {
        // Match / Switch
        valor: Expresion,
        casos: Vec<Caso>,
        defecto: Option<Vec<Sentencia>>,
    },

    // Saltos
    Romper,
    Continuar,
    Retornar(Option<Expresion>),
    Lanzar(Expresion),
    Pasar,

    // Definiciones
    Funcion {
        nombre: String,
        params: Vec<Parametro>,
        cuerpo: Vec<Sentencia>,
        es_async: bool,
        decoradores: Vec<Expresion>,
    },
    Clase {
        nombre: String,
        herencia: Vec<String>,
        cuerpo: Vec<Sentencia>,
        decoradores: Vec<Expresion>,
    },

    // Manejo de Errores
    TryCatch {
        cuerpo: Vec<Sentencia>,
        capturas: Vec<Captura>,
        finalmente: Option<Vec<Sentencia>>,
    },

    // Contexto
    Con {
        items: Vec<(Expresion, Option<String>)>, // (expresion, alias)
        cuerpo: Vec<Sentencia>,
    },

    // Módulos
    Importar {
        modulo: String,
        alias: Option<String>,
    },
    DesdeImportar {
        modulo: String,
        elementos: Vec<(String, Option<String>)>, // (nombre, alias)
    },

    // Variables
    Global(Vec<String>),
    NoLocal(Vec<String>),

    // Debug
    Imprimir(Vec<Expresion>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parametro {
    pub nombre: String,
    pub tipo: Option<String>,
    pub valor_por_defecto: Option<Expresion>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Caso {
    pub patron: Patron,
    pub guarda: Option<Expresion>,
    pub cuerpo: Vec<Sentencia>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Patron {
    Literal(Literal),
    Identificador(String),
    Comodin, // _
    Lista(Vec<Patron>),
    // Se pueden agregar más patrones (estructuras, or, etc.)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Captura {
    pub tipo: Option<String>,     // Tipo de excepción
    pub variable: Option<String>, // variable donde se captura 'as e'
    pub cuerpo: Vec<Sentencia>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Programa {
    pub sentencias: Vec<Sentencia>,
}
