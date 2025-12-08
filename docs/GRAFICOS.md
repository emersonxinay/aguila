# Documentación del Módulo `graficos`

El módulo `graficos` proporciona funciones nativas para crear ventanas, dibujar formas primitivas, manejar texto y detectar entrada del teclado. Está optimizado para alto rendimiento y dibujo en tiempo real.

## Configuración y Control

### `graficos.ventana(ancho: Entero, alto: Entero, titulo: Texto)`
Inicializa una ventana gráfica.
- **ancho**: Ancho de la ventana en píxeles.
- **alto**: Alto de la ventana en píxeles.
- **titulo**: Título que aparece en la barra de la ventana.

### `graficos.tecla(nombre: Texto) -> Booleano`
Verifica si una tecla específica está presionada en ese instante (polled input).
- **nombre**: Nombre de la tecla. Valores soportados:
  - "Arriba", "Abajo", "Izquierda", "Derecha"
  - "Espacio", "Enter", "Escape"
  - "W", "A", "S", "D"
- **Retorna**: `verdadero` si la tecla está presionada, `falso` en caso contrario.

### `graficos.actualizar()`
Actualiza el contenido de la ventana con lo que se ha dibujado en el buffer. Debe llamarse al final de cada frame en un ciclo de animación. Mantiene un framerate estable (60 FPS).

### `graficos.limpiar(color: Entero)`
Limpia toda la pantalla rellenándola con un color sólido.
- **color**: Color en formato entero RGB (ej. `0xFFFFFF` o decimal). Si se pasa `0`, limpia a negro transparente.

### `graficos.abierta() -> Booleano`
Retorna `verdadero` mientras la ventana permanezca abierta y `falso` si el usuario intenta cerrarla (clic en X o ESC). Útil para controlar el ciclo principal `mientras`.

## Primitivas de Dibujo

### `graficos.punto(x: Entero, y: Entero, color: Entero)`
Dibuja un solo píxel.

### `graficos.linea(x1: Entero, y1: Entero, x2: Entero, y2: Entero, color: Entero)`
Dibuja una línea usando el algoritmo anti-aliasing de Xiaolin Wu para bordes suaves.

### `graficos.rectangulo(x: Entero, y: Entero, ancho: Entero, alto: Entero, color: Entero, relleno: Booleano)`
Dibuja un rectángulo.
- **relleno**: Si es `verdadero`, rellena el rectángulo. Si es `falso`, solo dibuja el borde.

### `graficos.poligono(puntos: Lista, color: Entero)`
Dibuja un polígono relleno definido por una lista de vértices.
- **puntos**: Lista de listas `[[x1, y1], [x2, y2], ...]`. Utiliza algoritmo Scanline para el relleno.

### `graficos.texto(x: Entero, y: Entero, mensaje: Texto, color: Entero)`
Dibuja texto en la pantalla usando una fuente de mapa de bits incorporada.

## Ejemplo Completo

```aguila
importar graficos

graficos.ventana(800, 600, "Demo Graficos")
x = 400

mientras graficos.abierta() {
    if graficos.tecla("Izquierda") { x = x - 2 }
    if graficos.tecla("Derecha") { x = x + 2 }

    graficos.limpiar(0) # Limpiar a negro
    graficos.rectangulo(x, 300, 50, 50, 16711680, verdadero) # Rojo
    graficos.actualizar()
}
```
