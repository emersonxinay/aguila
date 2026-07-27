# Manual de Referencia: Lenguaje Águila (v2.8.1)

Documentación completa del lenguaje de programación Águila.

---

##  Índice
1. [Introducción](#introducción)
2. [Sintaxis Básica](#sintaxis-básica)
   - [Variables y Constantes](#variables-y-constantes)
   - [Tipos de Datos](#tipos-de-datos)
3. [Operadores](#operadores)
4. [Estructuras de Control](#estructuras-de-control)
5. [Funciones](#funciones)
6. [Programación Orientada a Objetos](#programación-orientada-a-objetos)
7. [Módulos e Importaciones](#módulos-e-importaciones)
8. [Concurrencia](#concurrencia)
9. [Manejo de Errores](#manejo-de-errores)

---

## 1. Introducción

Águila es un lenguaje dinámico, interpretado (con JIT) y escrito en español. Su diseño prioriza la legibilidad y la simplicidad, inspirado en Python pero con bloques estilo C/Rust.

---

## 2. Sintaxis Básica

### Comentarios
```aguila
# Esto es un comentario de una línea
```

### Variables y Constantes
Se recomienda usar `let` para declarar variables nuevas.

```aguila
let nombre = "Águila"
let version = 2.75
let activo = verdadero
```

> **Nota:** Aunque Águila soporta asignación directa (`x = 10`), el uso de `let` ayuda a evitar la creación accidental de variables globales y es necesario para variables locales en funciones.

### Tipos de Datos

| Tipo | Ejemplo | Descripción |
| :--- | :--- | :--- |
| **Entero** | `42`, `-5` | Números enteros (32-bit/64-bit según contexto). |
| **Decimal** | `3.14`, `0.5` | Números de punto flotante (64-bit). |
| **Texto** | `"Hola"`, `'Mundo'` | Cadenas UTF-8 inmutables. |
| **Logico** | `verdadero`, `falso` | Valores booleanos. |
| **Nulo** | `nulo` | Ausencia de valor. |
| **Lista** | `[1, 2, "a"]` | Array ordenado y mutable. |
| **Diccionario** | `{"a": 1}` | Mapa clave-valor (Hash Map). |
| **Rango** | `0 hasta 10` | Generador de secuencia numéricas. |

#### Interpolación de Texto
Usa el prefijo `a` antes de las comillas:
```aguila
let user = "Dev"
imprimir(a"Hola, {user}!") 
```

---

## 3. Operadores

### Aritméticos
`+`, `-`, `*`, `/` (división decimal), `//` (división entera), `%` (módulo), `**` (potencia).

### Comparación
`==`, `!=`, `<`, `>`, `<=`, `>=`.

### Lógicos
*   `y` (AND)
*   `o` (OR)
*   `no` (NOT)

---

## 4. Estructuras de Control

### Condicionales (`si` / `sino`)
```aguila
si edad >= 18 {
    imprimir("Mayor de edad")
} sino si edad > 13 {
    imprimir("Adolescente")
} sino {
    imprimir("Niño")
}
```

### Bucle `mientras`
```aguila
let i = 0
mientras i < 5 {
    imprimir(i)
    i = i + 1
}
```

### Bucle `para`
Ideal para recorrer listas, diccionarios o rangos.

```aguila
# Recorrer rango
para i = 1 hasta 10 {
    imprimir(i)
}

# Recorrer lista
let frutas = ["Manzana", "Pera"]
para fruta en frutas {
    imprimir(fruta)
}
```

Palabras clave de control:
*   `romper`: Termina el bucle inmediatamente.
*   `continuar`: Salta a la siguiente iteración.

### Comprensión de Listas
Permite construir listas filtrando o transformando elementos de otra colección de forma concisa.

```aguila
let lista = [1, 2, 3, 4, 5]
let cuadrados = [x ** 2 para x en lista si x % 2 == 0]
# cuadrados será [4, 16]
```

---

## 5. Funciones

Las funciones son ciudadanos de primera clase.

```aguila
funcion sumar(a, b) {
    retornar a + b
}

# Funciones Anónimas (Lambdas)
let duplicar = lambda x: x * 2
let sumar = lambda a, b: a + b
```

### Recursión
Águila optimiza la recursión directa mediante JIT.

```aguila
funcion fib(n) {
    si n < 2 { retornar n }
    retornar fib(n-1) + fib(n-2)
}
```

---

## 6. Programación Orientada a Objetos

Águila usa un modelo de clases clásico con encapsulamiento profesional (explícito e implícito).

```aguila
clase Animal {
    publico nombre = ""
    _edad = 0  # Implícitamente Protegido

    publico funcion init(nombre, edad) {
        yo.nombre = nombre  # 'yo' equivale a 'self' o 'this'
        yo._edad = edad
    }

    privado funcion __secreto() {
        imprimir("Solo accesible desde adentro")
    }

    publico funcion hablar() {
        imprimir("...")
    }
}

clase Perro : Animal {     # Herencia con ':'
    publico funcion hablar() {
        imprimir("Guau!")
    }
}

let firulais = nuevo Perro("Firulais", 3)
firulais.hablar()
```

Los modificadores de visibilidad pueden ser `publico`, `privado` y `protegido`. Alternativamente, puedes usar las convenciones de `_` (protegido) y `__` (privado).

---

## 7. Módulos e Importaciones

### Módulos Estándar
```aguila
usar "mate"
usar "tiempo"
usar "json"
```

### Módulos Locales
Puedes importar otros archivos `.ag`.
```aguila
importar "mi_modulo" desde "./libs"
```

---

## 8. Concurrencia

### Hilos
```aguila
usar "thread"

funcion tarea() {
    imprimir("Ejecutando en hilo")
}

let t = thread.crear(tarea)
t.unir()
```

### Asincronía (Async/Await)
```aguila
funcion asincrona obtener_datos() {
    # simular espera...
    retornar "Datos"
}

funcion asincrona main() {
    let d = esperar obtener_datos()
}
```

---

## 9. Manejo de Errores

```aguila
intentar {
    let x = 10 / 0
} capturar error {
    imprimir(a"Ocurrió un error: {error}")
} finalmente {
    imprimir("Limpieza...")
}
```

---

### Palabras Reservadas (Referencia Rápida)
`si`, `sino`, `mientras`, `para`, `romper`, `continuar`, `funcion`, `retornar`, `clase`, `nuevo`, `importar`, `desde`, `verdadero`, `falso`, `nulo`, `yo`, `super`, `intentar`, `capturar`, `finalmente`, `lanzar`, `asincrono`, `esperar`.

---
<div align="center">
Águila v2.8.1 • <a href="https://aguila-lang.org">Simplicidad y Potencia</a>
</div>
