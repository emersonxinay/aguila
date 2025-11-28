# 🦅 Águila v2.3.0

**Lenguaje de programación en español con velocidad de Rust**

[![NPM Version](https://img.shields.io/npm/v/aguila-lang)](https://www.npmjs.com/package/aguila-lang)
[![VS Code Extension](https://img.shields.io/visual-studio-marketplace/v/aguila-lang.aguila-vscode)](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## 🚀 Instalación Rápida

```bash
# NPM (Recomendado)
npm install -g aguila-lang

# Verificar instalación
aguila --version
```

---

## ✨ Novedades v2.3.0

### 1️⃣ Asignación a Índices
```aguila
lista = [1, 2, 3, 4, 5]
lista[0] = 100
lista[4] = 500
# [100, 2, 3, 4, 500]
```

### 2️⃣ Palabra Clave `romper`
```aguila
mientras verdadero {
    x = leer("Número: ")
    si x == secreto {
        imprimir "¡Ganaste!"
        romper
    }
}
```

### 3️⃣ Métodos Optimizados
```aguila
numeros = [5, 2, 8, 1, 9, 3]
imprimir numeros.suma()     # 28
imprimir numeros.minimo()   # 1
imprimir numeros.maximo()   # 9
```

---

## 📚 Documentación

- **[Tutorial Completo](TUTORIAL.md)** - Aprende desde cero
- **[Comparación con Python](AGUILA_VS_PYTHON.md)** - Por qué Águila es más simple
- **[Documentación Completa](DOCUMENTACION.md)** - Referencia del lenguaje
- **[Guía de Contribución](CONTRIBUTING.md)** - Cómo contribuir

---

## 🎯 Ejemplo Rápido

```aguila
# Fibonacci optimizado
funcion fib(n) {
    si n <= 1 {
        retornar n
    }
    retornar fib(n - 1) + fib(n - 2)
}

para i = 0 hasta 10 {
    imprimir fib(i)
}
```

---

## 🛠️ Desarrollo

```bash
# Clonar repositorio
git clone https://github.com/emersonxinay/aguila.git
cd aguila/aguila

# Compilar
cargo build --release

# Ejecutar tests
./probar.sh
```

---

## 📦 Estructura del Proyecto

```
proyecto_nuevo_lenguaje/
├── aguila/              # Compilador e intérprete (Rust)
├── aguila-vscode/       # Extensión VS Code
├── npm/                 # Paquete NPM
├── docs/                # Documentación de releases
├── README.md            # Este archivo
├── TUTORIAL.md          # Tutorial completo
├── DOCUMENTACION.md     # Referencia del lenguaje
└── AGUILA_VS_PYTHON.md  # Comparación con Python
```

---

## 🌟 Características

- ✅ **Sintaxis en español** - Natural para hispanohablantes
- ✅ **Velocidad de Rust** - Compilado a código nativo
- ✅ **Inferencia de tipos** - Sin conversiones manuales
- ✅ **OOP completo** - Clases, herencia, métodos
- ✅ **Módulos nativos** - JSON, FS, Math
- ✅ **REPL interactivo** - Prueba código al instante

---

## 📊 Comparación

| Característica | Python | Águila |
|---|---|---|
| Sintaxis | Inglés | **Español** |
| Velocidad | Interpretado | **Compilado (Rust)** |
| Conversiones | Manual (`int()`) | **Automática** |
| Bucles | `range(1, 10)` | **`para i = 1 hasta 10`** |
| Break | `break` | **`romper`** |

---

## 🤝 Contribuir

¡Las contribuciones son bienvenidas! Lee [CONTRIBUTING.md](CONTRIBUTING.md) para más detalles.

---

## 📄 Licencia

MIT © Emerson Espinoza

---

## 🔗 Enlaces

- **NPM:** https://www.npmjs.com/package/aguila-lang
- **VS Code:** https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode
- **GitHub:** https://github.com/emersonxinay/aguila

---

**Hecho con ❤️ para la comunidad hispanohablante**
