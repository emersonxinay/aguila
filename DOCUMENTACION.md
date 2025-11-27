# 🦅 Documentación Oficial de Águila

Bienvenido a la documentación oficial de **Águila**, un lenguaje de programación multiparadigma, expresivo y robusto, diseñado completamente en **español**. Águila combina la simplicidad de sintaxis inspirada en Python con la potencia de un tipado gradual y compilación a JavaScript moderno.

---

## 🚀 1. Instalación y Uso

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

---

## 📝 2. Sintaxis Básica

### Comentarios
```aguila
# Esto es un comentario de una línea
// Esto también es un comentario de una línea
```

### Variables y Constantes
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

### Condicionales (`si` / `sino`)
```aguila
edad = 18

si edad >= 18 {
    imprimir "Eres mayor de edad"
} sino {
    imprimir "Eres menor de edad"
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
