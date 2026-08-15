# Novedades en Águila v2.9.1 y Guía de Accesibilidad

La versión **2.9.1** de Águila incluye un salto significativo en la preparación del motor para el despliegue en producción (generación de compiladores) y, lo más importante, refina nuestro compromiso con la **Accesibilidad (A11Y)** para desarrolladores ciegos.

A continuación, detallamos las nuevas características, cómo usarlas y por qué son importantes.

---

## 1. Manejo de Excepciones: `intentar` y `capturar`

Antes, si un programa encontraba un error grave (como abrir un archivo que no existe), el intérprete se detenía bruscamente. Ahora, gracias a los bloques `intentar/capturar`, puedes anticipar y controlar esos errores sin que tu aplicación colapse.

**Sintaxis:**
```python
intentar {
    # Código que podría fallar
    x = 10 / 0
} capturar {
    # Código que se ejecuta si hay un error
    imprimir("Hubo un error matemático. Por favor, intenta de nuevo.")
} finalmente {
    # (Opcional) Código que siempre se ejecuta al final
    imprimir("Operación finalizada.")
}
```

**Beneficio para Accesibilidad:** Si desarrollas un juego interactivo de voz o una interfaz de consola, no quieres que tu programa "explote" dejándote con un error técnico que el Lector de Pantalla deba deletrear. Usando `capturar`, puedes atrapar el fallo y emitir un mensaje hablado amable ("Oye, el archivo de guardado no existe").

---

## 2. Gestión de Módulos: `importar`

Para organizar el código en múltiples archivos y conectarse a librerías externas de forma más semántica, hemos introducido `importar`.

**Sintaxis:**
```python
# Importar un archivo completo
importar matematicas

# Usar algo del módulo
x = matematicas.redondear(3.14)
```

*(Nota: La antigua palabra `usar` seguirá funcionando indefinidamente. Si tienes proyectos antiguos con `usar matemáticas`, no necesitas cambiar nada).*

---

## 3. Entrada Estándar: `leer()`

Capturar datos desde el teclado es fundamental para crear aplicaciones de consola interactivas. La función global `leer()` pausa el programa, espera a que el usuario escriba, y retorna el texto introducido al presionar Enter.

**Sintaxis:**
```python
imprimir("¿Cuál es tu nombre?")
nombre = leer()
imprimir("Hola, " + nombre + "!")
```

*(Nota: `ingresar()` también se mantiene como alias válido).*

**Beneficio para Accesibilidad:** La terminal interactiva (REPL) y la función `leer()` han sido optimizadas para no emitir caracteres especiales invisibles que interrumpan a los Lectores de Pantalla (NVDA, VoiceOver). El flujo de pregunta -> respuesta es completamente natural.

---

## 4. Control de Memoria para Videojuegos: `.limpiar()`

Cuando construyes juegos que deben correr a 60 FPS estables (como nuestro port de Mario Bros o Tetris), no puedes dejar que la basura se acumule en la RAM.

Hemos añadido el método `.limpiar()` a las colecciones nativas (`Lista` y `Diccionario`). Esto destruye las referencias internas de manera manual e instantánea, aliviando la carga del Garbage Collector.

**Sintaxis:**
```python
nivel_enemigos = [enemigo1, enemigo2, enemigo3]

# Al pasar de nivel, en lugar de esperar al GC, limpiamos la memoria:
nivel_enemigos.limpiar() 
```

---

## ♿ Compromiso con la Accesibilidad (A11Y)

Águila es pionero en ser un lenguaje compilado **diseñado en español con los lectores de pantalla en mente**. 

- **Errores Humanos:** Los mensajes de "Crash" o "Panic" del compilador están formateados con espacios e indentaciones que JAWS y NVDA interpretan correctamente, saltando verbosidad innecesaria.
- **Sin Ruido Visual:** Al ser un lenguaje basado en indentación con bloques clave muy claros (`si`, `mientras`), los desarrolladores invidentes no tienen que luchar contra llaves `{}` superpuestas ni puntos y comas `;` silenciosos al navegar su código.
- **Terminal Preparada:** El REPL de Águila limpia su buffer y posiciona el cursor de forma absoluta, evitando que el lector repita las líneas anteriores cada vez que presionas enter.
