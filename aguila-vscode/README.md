# 🦅 ÁGUILA - Extensión para VS Code

Soporte oficial de VS Code para el lenguaje de programación **ÁGUILA**, un lenguaje moderno en español diseñado para ser intuitivo y educativo.

## ✨ Características

- 🎨 **Resaltado de sintaxis completo** para archivos `.ag`
- 🔧 **Autocompletado** de palabras clave y métodos nativos
- 📁 **Icono personalizado** para archivos ÁGUILA
- 🔄 **Auto-cierre** de paréntesis, llaves y corchetes
- 💬 **Comentarios** con `#`
- 🌈 **Soporte para interpolación de strings** con `f"..."`

## 📦 Instalación

Busca "Aguila" en el Marketplace de VS Code o instala desde la terminal:

```bash
code --install-extension aguila-lang.aguila-vscode
```

## 🚀 Uso Rápido

Crea un archivo con extensión `.ag` y comienza a programar:

```aguila
# Hola Mundo
imprimir "¡Hola, mundo!"

# Operadores aritméticos
potencia = 2 ** 3  # 8 (nuevo en v2.2.1: ** en lugar de ^)
division_entera = 10 // 3  # 3

# Estructuras de datos
numeros = [1, 2, 3, 4, 5]
numeros.agregar(6)
imprimir numeros.longitud()  # 6

# Conjuntos (nuevo en v2.2.0)
s1 = conjunto([1, 2, 3])
s2 = conjunto([3, 4, 5])
union = s1.unir(s2)
imprimir union  # #{1, 2, 3, 4, 5}
```

## 🆕 Novedades en v2.6.1

- ✨ **Sintaxis Más Natural:** Se introduce `imprime` como la palabra clave principal (alias de `imprimir`).
- 🐛 **Correcciones de CI:** Mejoras en la estabilidad de los tests de integración.
- ⚡ **Optimización:** Mejoras en la gestión de memoria y rendimiento.

## ⌨️ Snippets y Atajos

### Snippets Disponibles
Escribe el prefijo y presiona Tab para expandir:

- `fun` → Función completa
- `si` → Condicional si
- `sisi` → Si-sino
- `para` → Bucle para-en
- `mientras` → Bucle mientras
- `clase` → Clase con constructor
- `try` → Intentar-capturar
- `segun` → Switch/match
- `imp` → imprimir
- `dict` → Diccionario
- `conjunto` → Conjunto

### Atajos de Teclado
- **Ctrl+/** o **Cmd+/** → Comentar/descomentar línea con `#`
- **Ctrl+K Ctrl+C** → Comentar selección
- **Ctrl+K Ctrl+U** → Descomentar selección
- **Enter** en comentario → Auto-continúa con `# `

## 📚 Sintaxis Soportada

### Palabras Clave
- **Control de flujo:** `si`, `sino`, `mientras`, `para`, `en`, `hasta`, `segun`, `caso`, `defecto`
- **Funciones:** `funcion`, `retornar`, `asincrono`, `esperar`
- **Clases:** `clase`, `nuevo`, `this`
- **Módulos:** `importar`
- **Errores:** `intentar`, `capturar`
- **Constantes:** `verdadero`, `falso`, `nulo`

### Tipos de Datos
- `Numero`, `Texto`, `Logico`, `Lista`, `Diccionario`, `Conjunto`

### Operadores
- **Aritméticos:** `+`, `-`, `*`, `/`, `//` (división entera), `%` (módulo), `**` (potencia)
- **Comparación:** `==`, `!=`, `>`, `<`, `>=`, `<=`
- **Lógicos:** `y`, `o`, `no`
- **Asignación:** `=`, `+=`, `-=`

### Métodos Nativos

**Listas:**
`.agregar()`, `.eliminar()`, `.insertar()`, `.longitud()`, `.contiene()`, `.ordenar()`, `.invertir()`, `.limpiar()`, `.copiar()`, `.unir()`, `.sublista()`

**Diccionarios:**
`.claves()`, `.valores()`, `.longitud()`, `.contiene()`, `.obtener()`, `.eliminar()`, `.limpiar()`, `.copiar()`

**Conjuntos (Sets):**
`.agregar()`, `.eliminar()`, `.contiene()`, `.longitud()`, `.unir()`, `.intersectar()`, `.diferencia()`, `.a_lista()`

**Texto:**
`.longitud()`, `.mayusculas()`, `.minusculas()`, `.contiene()`, `.reemplazar()`, `.dividir()`, `.recortar()`

### Funciones Globales
- `imprime()` / `imprimir()` - Imprime en consola
- `leer()` - Lee entrada del usuario (con inferencia de tipos)
- `afirmar()` - Aserciones para testing
- `conjunto()` - Crea un conjunto

## 🔗 Enlaces

- [Repositorio en GitHub](https://github.com/emersonxinay/aguila)
- [Documentación completa](https://github.com/emersonxinay/aguila/blob/main/DOCUMENTACION.md)
- [Reportar un problema](https://github.com/emersonxinay/aguila/issues)

## 📝 Licencia

MIT © 2025  [Emerson Espinoza](https://github.com/emersonxinay)
