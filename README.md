# 🦅 Águila 

> **"La simplicidad de Python. La velocidad de Rust. Todo en Español."**

<img src="aguila-vscode/icon.png" alt="Icono de Águila" width="100" height="100">


[![NPM Version](https://img.shields.io/npm/v/aguila-lang)](https://www.npmjs.com/package/aguila-lang)
[![VS Code Extension](https://img.shields.io/visual-studio-marketplace/v/aguila-lang.aguila-vscode)](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## 💡 ¿Por qué Águila?

Águila es un lenguaje de programación moderno diseñado para eliminar la barrera del idioma. Combina la sintaxis amigable de Python con el rendimiento de un lenguaje compilado como Rust.

*   **Código Nativo:** Escribe `si`, `mientras`, `funcion`. Piensa y programa en tu idioma.
*   **Rendimiento Real:** Compilado a código máquina. Es rápido, eficiente y robusto.
*   **Curva de Aprendizaje Cero:** Si conoces Python, ya sabes Águila.

---

## 🚀 Empezar es Fácil

### 1. Instala el Lenguaje
```bash
npm install -g aguila-lang
```

### 2. Instala la Extensión (Recomendado)
Para la mejor experiencia, instala la extensión oficial en **Visual Studio Code**.
*   🎨 Resaltado de sintaxis completo
*   ✨ Autocompletado inteligente
*   ⚡ Snippets de código

[**👉 Instalar Extensión desde Marketplace**](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)

---

## 📘 Tour de Sintaxis

Águila es expresivo y potente. Mira lo que puedes hacer:

### Variables y Tipos
```aguila
# Inferencia de tipos (Dinámico)
nombre = "Águila"
version = 2.4

# Tipado Estático (Opcional)
contador: Numero = 0
activo: Logico = verdadero
```

### Control de Flujo
```aguila
si edad >= 18 {
    imprimir "Eres mayor de edad"
} sino {
    imprimir "Eres menor"
}

# Bucles naturales
para i = 1 hasta 10 {
    imprimir f"Contando: ${i}"
}

mientras activo {
    romper  # Salir del bucle
}
```

### Funciones
```aguila
funcion saludar(nombre) {
    retornar f"Hola, ${nombre}!"
}

imprimir saludar("Mundo")
```

### Estructuras de Datos
```aguila
# Listas
frutas = ["Manzana", "Banana", "Uva"]
frutas.agregar("Naranja")
frutas[0] = "Pera"

# Diccionarios
usuario = {
    "nombre": "Emerson",
    "rol": "Admin"
}
imprimir usuario.obtener("nombre")
```

---

## � Potencia Algorítmica

**¿Es Águila un "juguete"? Definitivamente NO.**

Águila tiene **paridad lógica del 100% con Python**. Todo lo que puedes resolver en una entrevista técnica o en LeetCode con Python, puedes hacerlo en Águila.

| Nivel | Conceptos | Estado en Águila |
| :--- | :--- | :--- |
| **Básico** | Bucles, Condicionales, Matemáticas | ✅ Idéntico a Python |
| **Intermedio** | Listas, Diccionarios, Ordenamiento | ✅ Nativo y Optimizado |
| **Avanzado** | Recursión, Backtracking, Grafos (BFS/DFS) | ✅ Soporte Completo (v2.4) |

> **Dato:** Hemos verificado algoritmos complejos como *N-Queens*, *Sudoku Solver* y *Árboles Binarios* corriendo nativamente en Águila.

---

## 🆚 Comparativa: Python vs Águila

El mismo poder, en tu idioma.

| Característica | Python | Águila |
| :--- | :--- | :--- |
| Definir función | `def suma(a, b):` | `funcion suma(a, b) {` |
| Condicional | `if x > 0:` | `si x > 0 {` |
| Bucle | `for i in range(10):` | `para i = 0 hasta 10 {` |
| Imprimir | `print("Hola")` | `imprimir("Hola")` |
| Break | `break` | `romper` |

---

## 📚 Documentación y Recursos

*   🎓 **[Tutorial Paso a Paso](docs/tutorial.md):** Aprende desde cero con ejemplos.
*   📘 **[Manual de Referencia](docs/manual.md):** Documentación técnica completa.
*   🐍 **[Guía para Pythonistas](docs/vs_python.md):** Migra tus conocimientos.

---

## 🤝 Comunidad y Contribución

Águila es un proyecto de **Código Abierto** hecho con ❤️ para la comunidad global.

*   ¿Encontraste un bug? [Repórtalo en GitHub](https://github.com/emersonxinay/aguila/issues).
*   ¿Quieres contribuir? Lee nuestra [Guía de Contribución](CONTRIBUTING.md).

---

### 📄 Licencia
MIT © [Emerson Espinoza](https://github.com/emersonxinay)
