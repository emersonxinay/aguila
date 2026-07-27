# Glosario y Comparativa: Águila vs Python 

Este documento sirve como guía rápida para desarrolladores que vienen de Python. Águila comparte mucha filosofía con Python, pero con sintaxis traducida al español y algunas diferencias clave.

## Palabras Clave (Keywords)

| Águila  | Python  | Descripción |
| :--- | :--- | :--- |
| `funcion` / `fn` | `def` | Definición de función. |
| `retornar` | `return` | Retorno de valor. |
| `si` | `if` | Condicional. |
| `sino` | `else` | Rama alternativa del condicional. |
| `mientras` | `while` | Bucle mientras condición sea verdadera. |
| `para` | `for` | Bucle iterativo. |
| `en` | `in` | Pertenencia o iteración. |
| `romper` | `break` | Salir del bucle actual. |
| `continuar` | `continue` | Saltar a la siguiente iteración. |
| `clase` | `class` | Definición de clase. |
| `nuevo` | `__init__` (implícito) | Instanciación explicita (`nuevo Clase()`). |
| `verdadero` | `True` | Booleano verdadero. |
| `falso` | `False` | Booleano falso. |
| `nulo` | `None` | Valor nulo/vacío. |
| `y` | `and` | Operador lógico AND. |
| `o` | `or` | Operador lógico OR. |
| `no` | `not` | Operador lógico NOT / Negación. |
| `importar` | `import` | Importar módulos. |
| `desde` ... `importar` | `from` ... `import` | Importar específico. |
| `intentar` | `try` | Inicio bloque manejo de errores. |
| `capturar` | `except` | Captura de excepción. |
| `finalmente` | `finally` | Bloque de limpieza. |
| `lanzar` | `raise` | Lanzar un error manual. |
| `global` | `global` | Declarar variable global. |
| `eliminar` | `del` | Eliminar variable o propiedad. |
| `asincrono` | `async` | Función asíncrona. |
| `esperar` | `await` | Esperar promesa. |
| `segun` / `caso` | `match` / `case` | Pattern matching (switch). |
| `imprimir` | `print` | Imprimir en consola. |
| `let` | (no existe) | Declaración de variable (Águila requiere `let` inicial). |

## Tipos de Datos Principales

| Tipo Águila | Tipo Python | Notas |
| :--- | :--- | :--- |
| `Entero` | `int` | Números sin decimales. |
| `Decimal` | `float` | Números con punto flotante (64-bit). |
| `Texto` | `str` | Cadenas de caracteres (UTF-8). |
| `Lista` | `list` | Arreglo dinámico `[1, 2, 3]`. |
| `Diccionario` | `dict` | Mapa clave-valor `{"a": 1}`. |
| `Logico` | `bool` | `verdadero` o `falso`. |

## Diferencias Clave en Lógica

### 1. Declaración de Variables
*   **Python:** Implícita (`x = 10`)
*   **Águila:** Explícita (`let x = 10`). Esto ayuda a evitar errores de variables globales accidentales y mejora el rendimiento.

### 2. Bloques de Código
*   **Python:** Identación significativa (espacios/tabs).
*   **Águila:** Llaves `{ ... }`. Águila ignora la identación, dando libertad de formato (similar a JS/Rust/C).

### 3. Interpolación de Cadenas
*   **Python:** F-Strings (`f"Hola {nombre}"`).
*   **Águila:** A-Strings (`a"Hola {nombre}"`).

### 4. Instanciación
*   **Python:** `objeto = MiClase()`
*   **Águila:** `let objeto = nuevo MiClase()` (palabra clave `nuevo` es opcional en versiones recientes pero recomendada para claridad).

## Ejemplo Comparativo

**Python:**
```python
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

print(f"Resultado: {factorial(5)}")
```

**Águila:**
```aguila
funcion factorial(n) {
    si n <= 1 {
        retornar 1
    }
    retornar n * factorial(n - 1)
}

imprimir(a"Resultado: {factorial(5)}")
```
