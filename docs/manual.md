# 🦅 Documentación Oficial de Águila (v2.6.0)

Bienvenido a la documentación oficial de **Águila**, un lenguaje de programación diseñado para la educación, con sintaxis en español y alto rendimiento.

---

## 🚀 1. Instalación y Uso

### REPL (Consola Interactiva)
Ejecuta `aguila` en tu terminal para abrir la consola interactiva:
```bash
$ aguila
ÁGUILA v2.6.0
> imprimir("Hola Mundo")
"Hola Mundo"
```

### Ejecutar Archivos
Guarda tu código con extensión `.ag` y ejecútalo:
```bash
aguila mi_programa.ag
```

---

## 📝 2. Sintaxis Básica

Águila utiliza una sintaxis híbrida: palabras clave en español (inspiración Python) y bloques delimitados por llaves `{}` (estilo C/Rust).

### Variables
```rust
nombre = "Águila"
version = 2.6
es_rapido = verdadero
```

### Tipos de Datos
*   **Numero**: `10`, `3.14`, `-5`
*   **Texto**: `"Hola"`, `'Mundo'`
*   **Logico**: `verdadero`, `falso`
*   **Nulo**: `nulo`

### Listas
Colecciones ordenadas de elementos.
```rust
numeros = [1, 2, 3]
imprimir(numeros[0])  # Acceso: 1
numeros[1] = 99       # Modificación
```

### Diccionarios
Colecciones de pares clave-valor.
```rust
usuario = {"nombre": "Juan", "edad": 30}
imprimir(usuario["nombre"])  # Acceso: Juan
usuario["edad"] = 31         # Modificación
```

---

## 🔄 3. Estructuras de Control

### Condicionales
```rust
si edad >= 18 {
    imprimir("Mayor de edad")
} sino {
    imprimir("Menor de edad")
}
```

### Bucles
```rust
# Bucle Mientras
contador = 0
mientras contador < 5 {
    imprimir(contador)
    contador = contador + 1
}

# Bucle Para (Rangos)
para i = 0 hasta 5 {
    imprimir(i)
}
```

---

## 📦 4. Funciones

```rust
funcion sumar(a, b) {
    retornar a + b
}

resultado = sumar(5, 10)
imprimir(resultado)
```

---

## ⚡ 5. Asincronía (Nuevo en v2.6.0)

Águila soporta programación asíncrona básica con `asincrono` y `esperar`.

```rust
funcion asincrona tarea_lenta() {
    # ... lógica asíncrona ...
    retornar "Datos"
}

funcion asincrona main() {
    resultado = esperar tarea_lenta()
    imprimir(resultado)
}
```

---

## ⚠️ 6. Manejo de Errores

```rust
intentar {
    lanzar "Algo salió mal"
} capturar error {
    imprimir("Error capturado: " + error)
} finalmente {
    imprimir("Esto siempre se ejecuta")
}
```

---

## 🏛️ 7. Clases y Objetos

```rust
clase Persona {
    funcion init(nombre) {
        yo.nombre = nombre
    }

    funcion saludar() {
        imprimir("Hola, soy " + yo.nombre)
    }
}

p = Persona("Maria")
p.saludar()
```

---

## 📚 8. Biblioteca Estándar

### Funciones Globales
*   **`imprimir(valor)`**: Muestra valor en consola.
*   **`leer(mensaje)`**: Lee entrada del usuario.
*   **`afirmar(condicion, msg)`**: Lanza error si la condición es falsa.
*   **`reloj()`**: Devuelve tiempo actual en segundos.

### Módulos (Experimental)
*   **`net`**: Funciones de red (TCP).
*   **`mate`**: Funciones matemáticas.
*   **`lista`**: Utilidades para listas.

---

<div align="center">
Hecho con ❤️ por Emerson Espinoza
</div>
