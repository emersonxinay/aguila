# 🦅 ÁGUILA

> **"Lo mismo que Python, pero más veloz y en Español."**

![Versión](https://img.shields.io/badge/versión-v2.2.8-blue)

ÁGUILA es un lenguaje de programación multiparadigma (interpretado y compilado a JS), diseñado para ser expresivo, robusto y completamente en **español**.

## 🚀 Instalación

```bash
npm install -g aguila-lang
```

### Extensión de VS Code

Instala la extensión oficial para obtener resaltado de sintaxis y soporte completo:

**Opción 1: Desde VS Code Marketplace**
1. Abre VS Code
2. Ve a Extensions (Ctrl+Shift+X)
3. Busca "ÁGUILA"
4. Click en "Install"

**Opción 2: Desde la terminal**
```bash
code --install-extension aguila-lang.aguila-vscode
```

**Características de la extensión:**
- ✅ Resaltado de sintaxis para archivos `.ag`
- ✅ Icono personalizado en el explorador
- ✅ Auto-cierre de brackets y paréntesis
- ✅ Soporte para comentarios con `//`


## 🛠️ Herramientas (CLI)

ÁGUILA incluye un set completo de herramientas:

*   **`aguila repl`**: Consola interactiva para probar código rápidamente.
*   **`aguila ejecutar <archivo.ag>`**: Ejecuta scripts directamente con el intérprete nativo (Rust).
*   **`aguila compilar <archivo.ag>`**: Transpila tu código a JavaScript moderno (ES6) para correr en Node.js o navegadores.
*   **`aguila chequear <archivo.ag>`**: Analizador estático que busca errores antes de ejecutar (variables no definidas, tipos incorrectos, etc.).

## 📝 Sintaxis y Características

### 1. Variables y Tipos
Tipado dinámico pero con soporte opcional para tipos estáticos.
```aguila
# Inferencia de tipos
nombre = "Emerson"
edad = 25

# Tipos explícitos (verificados por 'aguila chequear')
activo: Logico = verdadero
pi: Numero = 3.1416
```

### 2. Estructuras de Control
```aguila
si edad >= 18 {
    imprimir "Mayor de edad"
} sino {
    imprimir "Menor de edad"
}

mientras activo {
    imprimir "Esperando..."
    activo = falso
}
```

### 3. Funciones
```aguila
funcion sumar(a, b) {
    retornar a + b
}

resultado = sumar(10, 20)
```

### 4. Módulos
Organiza tu código en múltiples archivos.
```aguila
# lib.ag
x = 42
module.exports = { x }

# main.ag
importar "./lib.ag" as lib
imprimir lib.x  # 42
```

### 5. Clases (POO)
Soporte completo para clases, herencia y constructores.
```aguila
clase Animal {
    nombre: Texto
    constructor(n) {
        this.nombre = n
    }
    hacer_sonido() {
        imprimir "..."
    }
}

clase Perro : Animal {
    hacer_sonido() {
        imprimir "Guau!"
    }
}

p = nuevo Perro("Firulais")
p.hacer_sonido()
```

### 6. Manejo de Errores
```aguila
intentar {
    x = 1 / 0
} capturar error {
    imprimir "Ocurrió un error: " + error
}
```

### 7. Biblioteca Estándar
Módulos nativos potentes integrados.

*   **`fs`**: Sistema de archivos (`fs.leer`, `fs.escribir`).
*   **`json`**: Parsing y stringify (`json.parsear`, `json.stringificar`).
*   **`red`**: Servidores TCP y HTTP (`red.servidor`).
*   **`mate`**: Funciones matemáticas (`mate.sin`, `mate.cos`, `mate.raiz`, `mate.aleatorio`).
*   **`fecha`**: Manejo de fechas (`fecha.ahora`, `fecha.formato`).

### 8. Algoritmos Avanzados (Nuevo en v2.2.8)
ÁGUILA está optimizado para ejecutar algoritmos complejos y estructuras de datos avanzadas.

```aguila
# Fibonacci con Memoización
memo = {}
funcion fib(n) {
    si n <= 1 { retornar n }
    clave = n.a_texto()
    si memo.contiene(clave) { retornar memo.obtener(clave) }
    res = fib(n-1) + fib(n-2)
    memo.insertar(clave, res)
    retornar res
}
imprimir fib(50) # Ultra rápido
```

### 9. Programación Asíncrona
Soporte nativo para async/await:
```aguila
asincrono funcion obtener_datos() {
    respuesta = esperar fetch("https://api.ejemplo.com/datos")
    retornar respuesta
}

datos = esperar obtener_datos()
imprimir(datos)
```


## 🌐 Compilación a JavaScript
ÁGUILA puede compilarse a JavaScript moderno con soporte completo para async/await:
```bash
aguila compilar mi_programa.ag
node mi_programa.js
```

El código generado es JavaScript ES6+ optimizado y listo para producción.

## 📚 Recursos

- **Marketplace**: [Extensión VS Code](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)
- **Repositorio**: [GitHub](https://github.com/emersonxinay/aguila)
- **Documentación**: Ver carpeta `ejemplos/` para más casos de uso


---
Hecho con ❤️ Emerson Espinoza
