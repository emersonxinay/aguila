# 🦅 Tutorial Completo de Águila: De Cero a Experto

> **Proyecto Real:** Sistema de Biblioteca - Aprende Águila construyendo una aplicación completa de gestión de libros y préstamos.

---

## 📚 Índice

1. [Introducción y Configuración](#1-introducción-y-configuración)
2. [Variables y Tipos de Datos](#2-variables-y-tipos-de-datos)
3. [Operadores y Expresiones](#3-operadores-y-expresiones)
4. [Estructuras de Control](#4-estructuras-de-control)
5. [Funciones](#5-funciones)
6. [Listas y Diccionarios](#6-listas-y-diccionarios)
7. [Programación Orientada a Objetos](#7-programación-orientada-a-objetos)
8. [Persistencia de Datos (JSON + FS)](#8-persistencia-de-datos)
9. [Proyecto Final: Sistema Completo](#9-proyecto-final-sistema-completo)

---

## 1. Introducción y Configuración

### ¿Qué es Águila?

Águila es un lenguaje de programación moderno, expresivo y completamente en español. Diseñado para ser intuitivo y potente, combina la simplicidad de Python con características avanzadas como tipado gradual y programación orientada a objetos.

### Instalación

```bash
# Opción 1: NPM (Recomendado)
npm install -g aguila-lang

# Opción 2: Desde el código fuente
git clone https://github.com/emersonxinay/aguila.git
cd aguila/aguila
cargo build --release
```

### Tu Primer Programa

Crea un archivo `hola.ag`:

```aguila
imprimir "¡Bienvenido a Águila! 🦅"
```

Ejecútalo:

```bash
aguila hola.ag
```

### El REPL Interactivo

Para experimentar rápidamente:

```bash
aguila
```

---

## 2. Variables y Tipos de Datos

### Variables Dinámicas

En Águila, no necesitas declarar el tipo de una variable:

```aguila
# Variables básicas
nombre = "Ana García"
edad = 28
es_estudiante = verdadero
saldo = 1500.50

imprimir nombre
imprimir edad
```

### Tipado Opcional

Para mayor claridad y seguridad, puedes especificar tipos:

```aguila
titulo: Texto = "El Quijote"
paginas: Numero = 863
disponible: Logico = verdadero

imprimir titulo + " tiene " + paginas + " páginas"
```

### Tipos de Datos

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| `Numero` | Enteros y decimales | `42`, `3.14` |
| `Texto` | Cadenas de caracteres | `"Hola"` |
| `Logico` | Booleanos | `verdadero`, `falso` |
| `Lista` | Colecciones ordenadas | `[1, 2, 3]` |
| `Diccionario` | Pares clave-valor | `{"nombre": "Ana"}` |
| `Nulo` | Ausencia de valor | `nulo` |

### 🎯 Ejercicio 1: Variables de Biblioteca

Crea un archivo `ejercicio1.ag`:

```aguila
# Información de un libro
titulo = "Cien Años de Soledad"
autor = "Gabriel García Márquez"
isbn = "978-0307474728"
paginas = 417
disponible = verdadero

imprimir "=== INFORMACIÓN DEL LIBRO ==="
imprimir "Título: " + titulo
imprimir "Autor: " + autor
imprimir "ISBN: " + isbn
imprimir "Páginas: " + paginas
imprimir "Disponible: " + disponible
```

---

## 3. Operadores y Expresiones

### Operadores Aritméticos

```aguila
# Calculadora de multas de biblioteca
dias_retraso = 5
multa_por_dia = 2.50

multa_total = dias_retraso * multa_por_dia
imprimir "Multa total: $" + multa_total  # $12.50

# Operadores avanzados
precio_libro = 100
descuento = precio_libro // 10  # División entera: 10
precio_final = precio_libro - descuento
imprimir "Precio con descuento: $" + precio_final  # $90
```

### Operadores de Comparación

```aguila
edad_usuario = 16
edad_minima = 18

puede_registrarse = edad_usuario >= edad_minima
imprimir "¿Puede registrarse? " + puede_registrarse  # falso

# Comparaciones múltiples
stock = 5
imprimir stock > 0  # verdadero
imprimir stock == 0  # falso
```

### Operadores Lógicos

```aguila
tiene_credencial = verdadero
debe_multas = falso

puede_prestar = tiene_credencial y no debe_multas
imprimir "¿Puede pedir prestado? " + puede_prestar  # verdadero
```

### Interpolación de Cadenas

```aguila
nombre = "Carlos"
libros_prestados = 3

mensaje = a"Hola {nombre}, tienes {libros_prestados} libros prestados"
imprimir mensaje
# Salida: Hola Carlos, tienes 3 libros prestados
```

### 🎯 Ejercicio 2: Calculadora de Multas

```aguila
# ejercicio2.ag
nombre_usuario = "María López"
dias_retraso = 7
tarifa_diaria = 3.0

# Cálculo de multa
multa_base = dias_retraso * tarifa_diaria
recargo = multa_base * 0.1  # 10% de recargo
multa_total = multa_base + recargo

# Mostrar resultado
imprimir a"Usuario: {nombre_usuario}"
imprimir a"Días de retraso: {dias_retraso}"
imprimir a"Multa base: ${multa_base}"
imprimir a"Recargo (10%): ${recargo}"
imprimir a"Total a pagar: ${multa_total}"
```

---

## 4. Estructuras de Control

### Condicionales: `si` / `sino`

```aguila
# Verificar disponibilidad de libro
libros_disponibles = 3

si libros_disponibles > 0 {
    imprimir "✅ Libro disponible para préstamo"
    libros_disponibles = libros_disponibles - 1
} sino {
    imprimir "❌ No hay copias disponibles"
}

imprimir a"Quedan {libros_disponibles} copias"
```

### Condicionales Anidados

```aguila
edad = 15
tiene_permiso_padres = verdadero

si edad >= 18 {
    imprimir "Acceso completo a la biblioteca"
} sino si edad >= 13 y tiene_permiso_padres {
    imprimir "Acceso con permiso de padres"
} sino {
    imprimir "Acceso solo a sección infantil"
}
```

### Selección Múltiple: `según`

```aguila
categoria = 2

segun categoria {
    caso 1 {
        imprimir "📚 Ficción"
    }
    caso 2 {
        imprimir "📖 No Ficción"
    }
    caso 3 {
        imprimir "🔬 Ciencia"
    }
    defecto {
        imprimir "❓ Categoría desconocida"
    }
}
```

### Bucle `mientras`

```aguila
# Procesar lista de espera
personas_en_espera = 5

mientras personas_en_espera > 0 {
    imprimir a"Atendiendo... Quedan {personas_en_espera}"
    personas_en_espera = personas_en_espera - 1
}

imprimir "✅ Lista de espera procesada"
```

### Bucle `para` con Rango

```aguila
# Generar códigos de estantería
imprimir "Códigos de estantería:"

para i = 1 hasta 6 {
    codigo = a"EST-{i}"
    imprimir codigo
}
# Salida: EST-1, EST-2, EST-3, EST-4, EST-5
```

### 🎯 Ejercicio 3: Sistema de Categorías

```aguila
# ejercicio3.ag
edad_usuario = leer("¿Cuál es tu edad? ")
categoria_libro = leer("Categoría (1=Infantil, 2=Juvenil, 3=Adulto): ")

# Validación de acceso
puede_acceder = falso

si categoria_libro == 1 {
    puede_acceder = verdadero
} sino si categoria_libro == 2 {
    puede_acceder = edad_usuario >= 13
} sino si categoria_libro == 3 {
    puede_acceder = edad_usuario >= 18
}

si puede_acceder {
    imprimir "✅ Acceso permitido"
} sino {
    imprimir "❌ No tienes edad suficiente para esta categoría"
}
```

---

## 5. Funciones

### Funciones Básicas

```aguila
funcion saludar_usuario(nombre) {
    imprimir a"¡Bienvenido a la biblioteca, {nombre}!"
}

saludar_usuario("Pedro")
```

### Funciones con Retorno

```aguila
funcion calcular_multa(dias) {
    tarifa = 2.5
    retornar dias * tarifa
}

multa = calcular_multa(10)
imprimir a"Multa: ${multa}"  # Multa: $25.0
```

### Funciones con Tipos

```aguila
funcion calcular_descuento(precio: Numero, porcentaje: Numero) -> Numero {
    descuento = precio * (porcentaje / 100)
    retornar precio - descuento
}

precio_final = calcular_descuento(100, 15)
imprimir a"Precio con descuento: ${precio_final}"  # $85.0
```

### Funciones con Múltiples Parámetros

```aguila
funcion registrar_prestamo(usuario, libro, dias) {
    imprimir "=== REGISTRO DE PRÉSTAMO ==="
    imprimir a"Usuario: {usuario}"
    imprimir a"Libro: {libro}"
    imprimir a"Días permitidos: {dias}"
}

registrar_prestamo("Ana", "El Principito", 14)
```

### 🎯 Ejercicio 4: Biblioteca de Funciones

```aguila
# ejercicio4.ag

funcion validar_isbn(isbn) {
    longitud = isbn.longitud()
    retornar longitud == 13 o longitud == 10
}

funcion calcular_dias_retraso(fecha_devolucion, fecha_actual) {
    # Simplificado: asumimos que son números de días
    retraso = fecha_actual - fecha_devolucion
    
    si retraso > 0 {
        retornar retraso
    } sino {
        retornar 0
    }
}

funcion generar_reporte(nombre, libros_prestados, multa) {
    imprimir "╔════════════════════════════╗"
    imprimir "║   REPORTE DE USUARIO       ║"
    imprimir "╚════════════════════════════╝"
    imprimir a"Nombre: {nombre}"
    imprimir a"Libros activos: {libros_prestados}"
    imprimir a"Multa pendiente: ${multa}"
}

# Uso
isbn_valido = validar_isbn("9780307474728")
imprimir a"ISBN válido: {isbn_valido}"

dias_retraso = calcular_dias_retraso(15, 20)
imprimir a"Días de retraso: {dias_retraso}"

generar_reporte("Carlos Ruiz", 2, 15.50)
```

---

## 6. Listas y Diccionarios

### Listas

```aguila
# Lista de libros disponibles
libros = ["El Quijote", "Cien Años de Soledad", "1984"]

# Acceso por índice
imprimir libros[0]  # El Quijote

# Agregar elementos
libros.agregar("Rayuela")
imprimir libros.longitud()  # 4

# Iterar sobre lista
para libro en libros {
    imprimir a"📚 {libro}"
}
```

### Métodos de Listas

```aguila
numeros = [5, 2, 8, 1, 9]

# Ordenar
numeros.ordenar()
imprimir numeros  # [1, 2, 5, 8, 9]

# Invertir
numeros.invertir()
imprimir numeros  # [9, 8, 5, 2, 1]

# Verificar contenido
tiene_cinco = numeros.contiene(5)
imprimir tiene_cinco  # verdadero

# Sublista
primeros_tres = numeros.sublista(0, 3)
imprimir primeros_tres  # [9, 8, 5]
```

### Diccionarios

```aguila
# Información de un libro
libro = {
    "titulo": "El Principito",
    "autor": "Antoine de Saint-Exupéry",
    "año": 1943,
    "disponible": verdadero
}

# Acceso a valores
imprimir libro["titulo"]  # El Principito

# Modificar valores
libro["disponible"] = falso

# Agregar nuevos campos
libro.insertar("prestado_a", "María")
```

### Métodos de Diccionarios

```aguila
libro = {"titulo": "1984", "autor": "Orwell", "paginas": 328}

# Obtener claves
claves = libro.claves()
imprimir claves  # ["titulo", "autor", "paginas"]

# Obtener valores
valores = libro.valores()
imprimir valores  # ["1984", "Orwell", 328]

# Verificar existencia
tiene_isbn = libro.contiene("isbn")
imprimir tiene_isbn  # falso
```

### 🎯 Ejercicio 5: Catálogo de Libros

```aguila
# ejercicio5.ag

# Crear catálogo
catalogo = []

# Función para agregar libro
funcion agregar_libro(titulo, autor, isbn) {
    libro = {
        "titulo": titulo,
        "autor": autor,
        "isbn": isbn,
        "disponible": verdadero,
        "prestamos": 0
    }
    catalogo.agregar(libro)
    imprimir a"✅ Libro '{titulo}' agregado al catálogo"
}

# Función para buscar libro
funcion buscar_libro(titulo_buscar) {
    para libro en catalogo {
        si libro["titulo"] == titulo_buscar {
            retornar libro
        }
    }
    retornar nulo
}

# Función para listar todos
funcion listar_catalogo() {
    imprimir "\n=== CATÁLOGO DE BIBLIOTECA ==="
    contador = 1
    
    para libro en catalogo {
        estado = "✅ Disponible"
        si no libro["disponible"] {
            estado = "❌ Prestado"
        }
        
        imprimir a"{contador}. {libro['titulo']} - {libro['autor']} {estado}"
        contador = contador + 1
    }
}

# Uso del sistema
agregar_libro("El Quijote", "Cervantes", "978-1234567890")
agregar_libro("Cien Años de Soledad", "García Márquez", "978-0987654321")
agregar_libro("1984", "George Orwell", "978-1111111111")

listar_catalogo()

# Buscar un libro
libro_encontrado = buscar_libro("1984")
si libro_encontrado != nulo {
    imprimir a"\n📖 Encontrado: {libro_encontrado['titulo']} por {libro_encontrado['autor']}"
}
```

---

## 7. Programación Orientada a Objetos

### Clases Básicas

```aguila
clase Libro {
    titulo: Texto
    autor: Texto
    isbn: Texto
    disponible: Logico

    constructor(titulo, autor, isbn) {
        this.titulo = titulo
        this.autor = autor
        this.isbn = isbn
        this.disponible = verdadero
    }

    mostrar_info() {
        imprimir a"📚 {this.titulo}"
        imprimir a"   Autor: {this.autor}"
        imprimir a"   ISBN: {this.isbn}"
        
        estado = "Disponible"
        si no this.disponible {
            estado = "Prestado"
        }
        imprimir a"   Estado: {estado}"
    }

    prestar() {
        si this.disponible {
            this.disponible = falso
            imprimir "✅ Libro prestado exitosamente"
        } sino {
            imprimir "❌ El libro no está disponible"
        }
    }

    devolver() {
        this.disponible = verdadero
        imprimir "✅ Libro devuelto"
    }
}

# Uso
mi_libro = nuevo Libro("El Principito", "Saint-Exupéry", "978-0156012195")
mi_libro.mostrar_info()
mi_libro.prestar()
mi_libro.devolver()
```

### Herencia

```aguila
clase Usuario {
    nombre: Texto
    id: Numero
    activo: Logico

    constructor(nombre, id) {
        this.nombre = nombre
        this.id = id
        this.activo = verdadero
    }

    saludar() {
        imprimir a"Hola, soy {this.nombre}"
    }
}

clase Estudiante : Usuario {
    carrera: Texto
    semestre: Numero

    constructor(nombre, id, carrera, semestre) {
        this.nombre = nombre
        this.id = id
        this.activo = verdadero
        this.carrera = carrera
        this.semestre = semestre
    }

    mostrar_perfil() {
        imprimir "=== PERFIL DE ESTUDIANTE ==="
        imprimir a"Nombre: {this.nombre}"
        imprimir a"ID: {this.id}"
        imprimir a"Carrera: {this.carrera}"
        imprimir a"Semestre: {this.semestre}"
    }
}

# Uso
estudiante = nuevo Estudiante("Ana García", 12345, "Ingeniería", 5)
estudiante.saludar()
estudiante.mostrar_perfil()
```

### 🎯 Ejercicio 6: Sistema POO Completo

```aguila
# ejercicio6.ag

clase Libro {
    titulo: Texto
    autor: Texto
    isbn: Texto
    disponible: Logico
    veces_prestado: Numero

    constructor(titulo, autor, isbn) {
        this.titulo = titulo
        this.autor = autor
        this.isbn = isbn
        this.disponible = verdadero
        this.veces_prestado = 0
    }

    a_diccionario() {
        retornar {
            "titulo": this.titulo,
            "autor": this.autor,
            "isbn": this.isbn,
            "disponible": this.disponible,
            "veces_prestado": this.veces_prestado
        }
    }
}

clase Usuario {
    nombre: Texto
    id: Numero
    libros_prestados: Lista

    constructor(nombre, id) {
        this.nombre = nombre
        this.id = id
        this.libros_prestados = []
    }

    prestar_libro(libro) {
        si libro.disponible {
            libro.disponible = falso
            libro.veces_prestado = libro.veces_prestado + 1
            this.libros_prestados.agregar(libro)
            imprimir a"✅ '{libro.titulo}' prestado a {this.nombre}"
        } sino {
            imprimir a"❌ '{libro.titulo}' no está disponible"
        }
    }

    devolver_libro(libro) {
        libro.disponible = verdadero
        imprimir a"✅ '{libro.titulo}' devuelto por {this.nombre}"
    }

    mostrar_prestamos() {
        imprimir a"\n📚 Libros de {this.nombre}:"
        si this.libros_prestados.longitud() == 0 {
            imprimir "   (No tiene libros prestados)"
        } sino {
            para libro en this.libros_prestados {
                imprimir a"   - {libro.titulo}"
            }
        }
    }
}

# Crear biblioteca
libro1 = nuevo Libro("El Quijote", "Cervantes", "978-1234567890")
libro2 = nuevo Libro("1984", "Orwell", "978-0987654321")

usuario1 = nuevo Usuario("Carlos Ruiz", 1001)

# Simular préstamos
usuario1.prestar_libro(libro1)
usuario1.prestar_libro(libro2)
usuario1.mostrar_prestamos()

# Devolver un libro
usuario1.devolver_libro(libro1)
```

---

## 8. Persistencia de Datos

### Módulo FS (File System)

```aguila
# Escribir archivo
contenido = "Lista de libros:\n- El Quijote\n- 1984"
fs.escribir("libros.txt", contenido)

# Leer archivo
texto = fs.leer("libros.txt")
imprimir texto
```

### Módulo JSON

```aguila
# Convertir a JSON
libro = {
    "titulo": "El Principito",
    "autor": "Saint-Exupéry",
    "paginas": 96
}

texto_json = json.stringificar(libro)
imprimir texto_json
# {"titulo":"El Principito","autor":"Saint-Exupéry","paginas":96}

# Parsear JSON
libro_recuperado = json.parsear(texto_json)
imprimir libro_recuperado["titulo"]  # El Principito
```

### Guardar y Cargar Datos

```aguila
clase BibliotecaDB {
    archivo: Texto
    libros: Lista

    constructor(archivo) {
        this.archivo = archivo
        this.libros = []
        this.cargar()
    }

    cargar() {
        intentar {
            contenido = fs.leer(this.archivo)
            this.libros = json.parsear(contenido)
            imprimir a"✅ Cargados {this.libros.longitud()} libros"
        } capturar error {
            imprimir "ℹ️ No hay datos previos, iniciando nueva base"
            this.libros = []
        }
    }

    guardar() {
        texto = json.stringificar(this.libros)
        fs.escribir(this.archivo, texto)
        imprimir "💾 Datos guardados"
    }

    agregar_libro(libro_dict) {
        this.libros.agregar(libro_dict)
        this.guardar()
    }
}

# Uso
db = nuevo BibliotecaDB("biblioteca.json")
db.agregar_libro({
    "titulo": "Rayuela",
    "autor": "Cortázar",
    "isbn": "978-1111111111"
})
```

---

## 9. Proyecto Final: Sistema Completo

### Estructura del Proyecto

```
biblioteca/
├── main.ag              # Programa principal
├── modelos.ag           # Clases Libro y Usuario
├── database.ag          # Gestión de persistencia
└── biblioteca.json      # Datos (generado automáticamente)
```

### `modelos.ag`

```aguila
# modelos.ag

clase Libro {
    id: Numero
    titulo: Texto
    autor: Texto
    isbn: Texto
    disponible: Logico
    prestado_a: Texto

    constructor(id, titulo, autor, isbn) {
        this.id = id
        this.titulo = titulo
        this.autor = autor
        this.isbn = isbn
        this.disponible = verdadero
        this.prestado_a = ""
    }

    a_diccionario() {
        retornar {
            "id": this.id,
            "titulo": this.titulo,
            "autor": this.autor,
            "isbn": this.isbn,
            "disponible": this.disponible,
            "prestado_a": this.prestado_a
        }
    }

    prestar(usuario_nombre) {
        si this.disponible {
            this.disponible = falso
            this.prestado_a = usuario_nombre
            retornar verdadero
        }
        retornar falso
    }

    devolver() {
        this.disponible = verdadero
        this.prestado_a = ""
    }
}

clase Usuario {
    nombre: Texto
    id: Numero
    email: Texto

    constructor(nombre, id, email) {
        this.nombre = nombre
        this.id = id
        this.email = email
    }

    a_diccionario() {
        retornar {
            "nombre": this.nombre,
            "id": this.id,
            "email": this.email
        }
    }
}
```

### `main.ag` - Sistema Completo

```aguila
# main.ag - Sistema de Biblioteca Completo

archivo_db = "biblioteca.json"

clase Biblioteca {
    libros: Lista
    usuarios: Lista
    siguiente_id: Numero

    constructor() {
        this.libros = []
        this.usuarios = []
        this.siguiente_id = 1
        this.cargar_datos()
    }

    cargar_datos() {
        intentar {
            contenido = fs.leer(archivo_db)
            datos = json.parsear(contenido)
            this.libros = datos["libros"]
            this.usuarios = datos["usuarios"]
            this.siguiente_id = datos["siguiente_id"]
            imprimir "✅ Datos cargados correctamente"
        } capturar error {
            imprimir "ℹ️ Iniciando nueva biblioteca"
        }
    }

    guardar_datos() {
        datos = {
            "libros": this.libros,
            "usuarios": this.usuarios,
            "siguiente_id": this.siguiente_id
        }
        texto = json.stringificar(datos)
        fs.escribir(archivo_db, texto)
        imprimir "💾 Cambios guardados"
    }

    agregar_libro(titulo, autor, isbn) {
        libro = {
            "id": this.siguiente_id,
            "titulo": titulo,
            "autor": autor,
            "isbn": isbn,
            "disponible": verdadero,
            "prestado_a": ""
        }
        this.libros.agregar(libro)
        this.siguiente_id = this.siguiente_id + 1
        this.guardar_datos()
        imprimir a"✅ Libro '{titulo}' agregado con ID {libro['id']}"
    }

    listar_libros() {
        imprimir "\n╔════════════════════════════════════════╗"
        imprimir "║        CATÁLOGO DE BIBLIOTECA          ║"
        imprimir "╚════════════════════════════════════════╝"
        
        si this.libros.longitud() == 0 {
            imprimir "  (No hay libros en el catálogo)"
            retornar
        }

        para libro en this.libros {
            estado = "✅ Disponible"
            info_extra = ""
            
            si no libro["disponible"] {
                estado = "❌ Prestado"
                info_extra = a" (a {libro['prestado_a']})"
            }
            
            imprimir a"{libro['id']}. {libro['titulo']}"
            imprimir a"   📖 Autor: {libro['autor']}"
            imprimir a"   📋 ISBN: {libro['isbn']}"
            imprimir a"   {estado}{info_extra}"
            imprimir ""
        }
    }

    buscar_libro_por_id(id) {
        para libro en this.libros {
            si libro["id"] == id {
                retornar libro
            }
        }
        retornar nulo
    }

    prestar_libro(id_libro, nombre_usuario) {
        libro = this.buscar_libro_por_id(id_libro)
        
        si libro == nulo {
            imprimir "❌ Libro no encontrado"
            retornar
        }

        si libro["disponible"] {
            libro["disponible"] = falso
            libro["prestado_a"] = nombre_usuario
            this.guardar_datos()
            imprimir a"✅ '{libro['titulo']}' prestado a {nombre_usuario}"
        } sino {
            imprimir a"❌ El libro ya está prestado a {libro['prestado_a']}"
        }
    }

    devolver_libro(id_libro) {
        libro = this.buscar_libro_por_id(id_libro)
        
        si libro == nulo {
            imprimir "❌ Libro no encontrado"
            retornar
        }

        si no libro["disponible"] {
            libro["disponible"] = verdadero
            libro["prestado_a"] = ""
            this.guardar_datos()
            imprimir a"✅ '{libro['titulo']}' devuelto correctamente"
        } sino {
            imprimir "❌ Este libro no estaba prestado"
        }
    }

    estadisticas() {
        total = this.libros.longitud()
        disponibles = 0
        prestados = 0

        para libro en this.libros {
            si libro["disponible"] {
                disponibles = disponibles + 1
            } sino {
                prestados = prestados + 1
            }
        }

        imprimir "\n╔════════════════════════════════════════╗"
        imprimir "║          ESTADÍSTICAS                  ║"
        imprimir "╚════════════════════════════════════════╝"
        imprimir a"📚 Total de libros: {total}"
        imprimir a"✅ Disponibles: {disponibles}"
        imprimir a"❌ Prestados: {prestados}"
    }
}

# ========== PROGRAMA PRINCIPAL ==========

biblioteca = nuevo Biblioteca()
corriendo = verdadero

mientras corriendo {
    imprimir "\n╔════════════════════════════════════════╗"
    imprimir "║    SISTEMA DE BIBLIOTECA - ÁGUILA      ║"
    imprimir "╚════════════════════════════════════════╝"
    imprimir "1. Ver catálogo"
    imprimir "2. Agregar libro"
    imprimir "3. Prestar libro"
    imprimir "4. Devolver libro"
    imprimir "5. Estadísticas"
    imprimir "6. Salir"
    imprimir ""
    
    opcion = leer("Selecciona una opción: ")
    
    segun opcion {
        caso 1 {
            biblioteca.listar_libros()
        }
        caso 2 {
            titulo = leer("Título del libro: ")
            autor = leer("Autor: ")
            isbn = leer("ISBN: ")
            biblioteca.agregar_libro(titulo, autor, isbn)
        }
        caso 3 {
            biblioteca.listar_libros()
            id = leer("ID del libro a prestar: ")
            usuario = leer("Nombre del usuario: ")
            biblioteca.prestar_libro(id, usuario)
        }
        caso 4 {
            biblioteca.listar_libros()
            id = leer("ID del libro a devolver: ")
            biblioteca.devolver_libro(id)
        }
        caso 5 {
            biblioteca.estadisticas()
        }
        caso 6 {
            imprimir "\n¡Hasta luego! 👋"
            corriendo = falso
        }
        defecto {
            imprimir "❌ Opción no válida"
        }
    }
}
```

---

## 🎓 Conclusión

¡Felicidades! Has completado el tutorial completo de Águila construyendo un sistema real de biblioteca.

### Lo que has aprendido:

✅ Variables y tipos de datos  
✅ Operadores y expresiones  
✅ Estructuras de control (si/sino, según, mientras, para)  
✅ Funciones con parámetros y retorno  
✅ Listas y diccionarios  
✅ Programación orientada a objetos  
✅ Persistencia de datos con JSON y FS  
✅ Manejo de errores con intentar/capturar  
✅ Desarrollo de aplicaciones completas  

### Próximos Pasos

1. **Expande el proyecto:** Agrega funcionalidades como:
   - Sistema de multas por retraso
   - Búsqueda avanzada de libros
   - Historial de préstamos
   - Reportes en formato texto

2. **Explora características avanzadas:**
   - Funciones asíncronas (`asincrono`/`esperar`)
   - Módulos matemáticos (`mate`)
   - Conjuntos y operaciones de conjunto

3. **Comparte tu código:**
   - Publica en GitHub
   - Contribuye a la comunidad de Águila

---

## 📚 Recursos Adicionales

- [Documentación Oficial](https://github.com/emersonxinay/aguila)
- [Extensión VS Code](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)
- [Más Ejemplos](https://github.com/emersonxinay/aguila/tree/main/aguila/ejemplos)

---

**Hecho con ❤️ para la comunidad de Águila** 🦅
