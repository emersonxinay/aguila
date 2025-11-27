# 🦅 Documentación Oficial de Águila

Bienvenido a la documentación oficial de **Águila**, un lenguaje de programación multiparadigma, expresivo y robusto, diseñado completamente en **español**. Águila combina la simplicidad de sintaxis inspirada en Python con la potencia de un tipado gradual y compilación a JavaScript moderno.

---

## 🚀 Instalación y Uso

### REPL (Consola Interactiva)
Para probar el lenguaje rápidamente, simplemente ejecuta el comando `aguila` en tu terminal sin argumentos. Esto abrirá una consola interactiva donde puedes escribir código línea por línea.

```bash
$ aguila
ÁGUILA v2.1.2
Escribe 'salir' para terminar, o 'ayuda' para ver comandos.
> imprimir "Hola"
Hola
> 2 + 2
=> 4
```

### Ejecutar Archivos
Guarda tu código en un archivo con extensión `.ag` y ejecútalo:
```bash
aguila ejecutar mi_programa.ag
# O simplemente:
aguila mi_programa.ag
```

### Instalación
Para instalar Águila globalmente en tu sistema, necesitas tener Node.js instalado. Ejecuta el siguiente comando en tu terminal:

```bash
npm install -g aguila-lang
```

### Comandos CLI
Águila incluye una herramienta de línea de comandos (CLI) versátil:

*   **`aguila repl`**: Inicia una consola interactiva para probar código rápidamente.
*   **`aguila ejecutar <archivo.ag>`**: Interpreta y ejecuta un archivo `.ag` directamente.
*   **`aguila compilar <archivo.ag>`**: Compila el código Águila a JavaScript (ES6) optimizado.
*   **`aguila chequear <archivo.ag>`**: Realiza un análisis estático para detectar errores de tipos y variables antes de ejecutar.

### Entrada de Datos
Puedes solicitar información al usuario desde la consola usando la función `leer`. Esta función detecta automáticamente el tipo de dato ingresado (Número, Lógico o Texto).

```aguila
nombre = leer("¿Cómo te llamas? ")
edad = leer("¿Cuántos años tienes? ")  # Se convierte a Numero automáticamente
es_programador = leer("¿Programas? (verdadero/falso) ") # Se convierte a Logico

imprimir "Hola " + nombre
imprimir "En 10 años tendrás " + (edad + 10)
```

---

## 📝 2. Sintaxis Básica

### Comentarios
Usa `#` para comentarios de una sola línea.
*Nota: `//` ya no se usa para comentarios, ahora es el operador de división entera.*

```aguila
# Esto es un comentario
nombre = "Águila" # Comentario al final de línea
```

### Operadores

#### Aritméticos
| Operador | Descripción | Ejemplo |
|---|---|---|
| `+` | Suma | `10 + 5` (15) |
| `-` | Resta | `10 - 5` (5) |
| `*` | Multiplicación | `10 * 5` (50) |
| `/` | División | `10 / 3` (3.33...) |
| `//` | División Entera | `10 // 3` (3) |
| `%` | Módulo (Resto) | `10 % 3` (1) |
| `**` | Potencia | `2 ** 3` (8) |

#### Comparación
| Operador | Descripción | Ejemplo |
|---|---|---|
| `==` | Igual a | `5 == 5` (verdadero) |
| `!=` | Diferente de | `5 != 3` (verdadero) |
| `>` | Mayor que | `10 > 5` (verdadero) |
| `<` | Menor que | `5 < 10` (verdadero) |
| `>=` | Mayor o igual que | `5 >= 5` (verdadero) |
| `<=` | Menor o igual que | `3 <= 5` (verdadero) |

#### Lógicos
| Operador | Descripción | Ejemplo |
|---|---|---|
| `y` | AND lógico | `verdadero y falso` (falso) |
| `o` | OR lógico | `verdadero o falso` (verdadero) |
| `no` | NOT lógico | `no verdadero` (falso) |

#### Asignación
| Operador | Descripción | Ejemplo |
|---|---|---|
| `=` | Asignación simple | `a = 5` |
| `+=` | Suma y asigna | `a += 1` (a = a + 1) |
| `-=` | Resta y asigna | `a -= 1` (a = a - 1) |

### Variables y Tipos de Datos
Águila soporta tipado dinámico por defecto, pero permite tipado estático opcional para mayor seguridad.

**Tipado Dinámico:**
```aguila
nombre = "Águila"
version = 1.0
es_genial = verdadero
```

**Tipado Estático:**
```aguila
edad: Numero = 25
mensaje: Texto = "Hola Mundo"
activo: Logico = falso
```

### Tipos de Datos Primitivos
*   **Numero**: Enteros y flotantes (`10`, `3.14`, `-5`).
*   **Texto**: Cadenas de caracteres (`"Hola"`, `'Mundo'`).
*   **Logico**: Valores booleanos (`verdadero`, `falso`).
*   **Nulo**: Representa la ausencia de valor (`nulo`).

### Métodos Nativos

Águila incluye métodos integrados para manipular Listas y Textos fácilmente.

#### Métodos de Listas
| Método | Descripción | Ejemplo |
| :--- | :--- | :--- |
| `agregar(elemento)` | Añade un elemento al final. | `lista.agregar(4)` |
| `eliminar(indice)` | Elimina el elemento en el índice dado. | `lista.eliminar(0)` |
| `insertar(i, e)` | Inserta elemento `e` en índice `i`. | `lista.insertar(1, 5)` |
| `longitud()` | Devuelve la cantidad de elementos. | `lista.longitud()` |
| `contiene(e)` | Devuelve `verdadero` si `e` está en la lista. | `lista.contiene(2)` |
| `ordenar()` | Ordena la lista (números o textos). | `lista.ordenar()` |
| `invertir()` | Invierte el orden de la lista. | `lista.invertir()` |
| `limpiar()` | Elimina todos los elementos. | `lista.limpiar()` |
| `copiar()` | Devuelve una copia de la lista. | `l2 = l1.copiar()` |
| `unir(sep)` | Une elementos en un texto con separador. | `["a","b"].unir("-")` -> `"a-b"` |
| `sublista(i, f)` | Devuelve sublista desde `i` hasta `f` (excluido). | `l.sublista(0, 2)` |

#### Métodos de Texto
| Método | Descripción | Ejemplo |
|---|---|---|
| `longitud()` | Devuelve el largo del texto | `"hola".longitud()` |
| `mayusculas()` | Convierte a mayúsculas | `"hola".mayusculas()` |
| `minusculas()` | Convierte a minúsculas | `"HOLA".minusculas()` |
| `contiene(sub)` | Verifica si contiene el subtexto | `"hola".contiene("la")` |
| `reemplazar(a, b)` | Reemplaza `a` por `b` | `"hola".reemplazar("h", "H")` |
| `dividir(sep)` | Divide el texto en una lista | `"a,b".dividir(",")` |
| `recortar()` | Elimina espacios al inicio y final | `" a ".recortar()` |

### Testing Integrado
Águila incluye una función nativa para facilitar la creación de pruebas y verificar el correcto funcionamiento de tu código.

| Función | Descripción | Ejemplo |
|---|---|---|
| `afirmar(condicion, mensaje)` | Detiene el programa con un error si la condición es falsa. | `afirmar(x > 0, "x debe ser positivo")` |

### Interpolación de Cadenas
Puedes insertar expresiones dentro de cadenas de texto usando el prefijo `a` y llaves `{}`:

```aguila
nombre = "Usuario"
saludo = a"Hola, {nombre}. 2 + 2 es {2 + 2}"
imprimir saludo  # Salida: Hola, Usuario. 2 + 2 es 4
```

### Estructuras de Datos
**Listas:**
```aguila
numeros = [1, 2, 3, 4, 5]
mixta = [1, "dos", verdadero]
imprimir numeros[0]  # Acceso por índice
```

**Diccionarios:**
```aguila
usuario = {
    "nombre": "Emerson",
    "edad": 30
}
imprimir usuario["nombre"]
```

---

## 🔄 3. Estructuras de Control

### Condicionales (`si` / `sino si` / `sino`)
```aguila
edad = 18

si edad < 13 {
    imprimir "Eres un niño"
} sino si edad < 18 {
    imprimir "Eres un adolescente"
} sino {
    imprimir "Eres un adulto"
}
```

### Selección Múltiple (`según`)
Usa `según` para evaluar una expresión contra múltiples casos.

```aguila
opcion = 2

segun opcion {
    caso 1 {
        imprimir "Opción 1"
    }
    caso 2 {
        imprimir "Opción 2"
    }
    defecto {
        imprimir "Opción inválida"
    }
}
```

### Bucle `mientras`
```aguila
contador = 0
mientras contador < 5 {
    imprimir contador
    contador = contador + 1
}
```

### Bucle `para`
**Iterar sobre un rango:**
```aguila
# Imprime del 0 al 4
para i = 0 hasta 5 {
    imprimir i
}
```

**Iterar sobre una lista:**
```aguila
frutas = ["manzana", "banana", "uva"]
para fruta en frutas {
    imprimir fruta
}
```

---

## 📦 4. Funciones

### Definición Básica
```aguila
funcion saludar(nombre) {
    imprimir "Hola " + nombre
}

saludar("Mundo")
```

### Retorno de Valores y Tipos
Puedes especificar tipos de parámetros y de retorno opcionalmente:

```aguila
funcion sumar(a: Numero, b: Numero) -> Numero {
    retornar a + b
}

resultado = sumar(5, 10)
```

### Funciones Anónimas (Lambdas)
```aguila
operacion = funcion(x, y) {
    retornar x * y
}
imprimir operacion(3, 4)
```

### Funciones Asíncronas
Soporte nativo para `async/await` con las palabras clave `asincrono` y `esperar`.

```aguila
asincrono funcion obtener_datos() {
    # Simulación de operación asíncrona
    datos = esperar fetch("https://api.ejemplo.com")
    retornar datos
}
```

---

## 🏛️ 5. Programación Orientada a Objetos (POO)

Águila soporta clases, herencia, constructores y métodos.

### Definición de Clases
```aguila
clase Persona {
    nombre: Texto
    edad: Numero

    constructor(nombre, edad) {
        this.nombre = nombre
        this.edad = edad
    }

    saludar() {
        imprimir "Hola, soy " + this.nombre
    }
}

p = nuevo Persona("Juan", 25)
p.saludar()
```

### Herencia
Usa `:` para heredar de otra clase.

```aguila
clase Empleado : Persona {
    puesto: Texto

    constructor(nombre, edad, puesto) {
        # Nota: La llamada a super() es implícita o manual según implementación
        this.nombre = nombre
        this.edad = edad
        this.puesto = puesto
    }

    trabajar() {
        imprimir this.nombre + " está trabajando como " + this.puesto
    }
}
```

---

## 🧩 6. Módulos

Organiza tu código dividiéndolo en múltiples archivos.

**archivo `matematicas.ag`:**
```aguila
funcion duplicar(n) {
    retornar n * 2
}
# Todo lo definido es público por defecto o se exporta explícitamente (según implementación de runtime)
```

**archivo `main.ag`:**
```aguila
importar "./matematicas.ag" como mate

resultado = mate.duplicar(10)
imprimir resultado  # 20
```

---

## ⚠️ 7. Manejo de Errores

Usa bloques `intentar` y `capturar` para manejar excepciones de forma elegante.

```aguila
intentar {
    resultado = 10 / 0
} capturar error {
    imprimir "Ocurrió un error: " + error
}
```

---

## 📚 8. Biblioteca Estándar

Águila incluye módulos nativos potentes disponibles globalmente o via importación.

*   **`fs`**: Operaciones de sistema de archivos.
    *   `fs.leer(ruta)`
    *   `fs.escribir(ruta, contenido)`
*   **`json`**: Manipulación de JSON.
    *   `json.parsear(texto)`
    *   `json.stringificar(objeto)`
*   **`red`**: Funcionalidades de red (HTTP/TCP).
*   **`mate`**: Funciones matemáticas avanzadas (`sin`, `cos`, `raiz`, `aleatorio`).
*   **`fecha`**: Manejo de fechas y horas.

---

Hecho con ❤️ por el equipo de Águila.
