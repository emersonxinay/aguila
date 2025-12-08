# Documentación de la Librería Tortuga (Águila)

La librería `tortuga` proporciona una forma fácil y visual de aprender programación creando gráficos y dibujos. Permite controlar una "tortuga" en la pantalla que dibuja líneas a medida que se mueve.

## Inicio Rápido

Para usar la tortuga, primero debe importar el módulo y crear una instancia.

```aguila
importar tortuga
importar graficos # Necesario para abrir la ventana

# 1. Configurar la ventana
graficos.ventana(800, 600, "Mi Dibujo")

# 2. Crear una tortuga en el centro (x, y)
t = tortuga.Tortuga(400, 300)

# 3. Dibujar
t.avanzar(100)
t.derecha(90)
t.avanzar(100)

# 4. Mantener ventana abierta
mientras graficos.abierta() {
    graficos.actualizar()
}
```

## Referencia de API

### Constructor

`t = tortuga.Tortuga(x, y)`
Crea una nueva tortuga en la posición especificada.
- `x`: Posición horizontal inicial.
- `y`: Posición vertical inicial.

### Desplazamiento

- `t.avanzar(distancia)`: Mueve la tortuga hacia adelante en la dirección actual. Si la pluma está abajo, dibuja una línea.
- `t.retroceder(distancia)`: Mueve la tortuga hacia atrás.
- `t.ir_a(x, y)`: Mueve la tortuga instantáneamente a la posición `(x, y)`. Dibuja línea si la pluma está abajo.

### Giro

- `t.derecha(grados)`: Gira la tortuga hacia la derecha (sentido horario) el número de grados especificado.
- `t.izquierda(grados)`: Gira la tortuga hacia la izquierda (sentido antihorario).

### Control de Pluma y Color

- `t.pluma_arriba()`: Levanta la pluma. La tortuga se moverá sin dibujar.
- `t.pluma_abajo()`: Baja la pluma. La tortuga dibujará al moverse (por defecto).
- `t.poner_color(r, g, b)`: Cambia el color del trazo.
    - `r`, `g`, `b`: Componentes Rojo, Verde y Azul (0-255).
- `t.poner_relleno(r, g, b)`: Cambia el color de relleno para figuras cerradas.

### Relleno de Figuras

Para rellenar una forma, use `inicio_relleno` antes de empezar a dibujar y `fin_relleno` al terminar.

```aguila
t.poner_relleno(255, 0, 0) # Rojo
t.inicio_relleno()
t.cuadrado(100)
t.fin_relleno()
```

- `t.inicio_relleno()`: Comienza a registrar los puntos del polígono.
- `t.fin_relleno()`: Cierra el polígono y lo rellena con el color configurado.

### Figuras Geométricas

Funciones auxiliares para dibujar formas comunes rápidamente:

- `t.cuadrado(lado)`: Dibuja un cuadrado del tamaño especificado.
- `t.rectangulo(ancho, alto)`: Dibuja un rectángulo.
- `t.circulo(radio)`: Dibuja un círculo.
- `t.triangulo(lado)`: Dibuja un triángulo equilátero.

### Texto

- `t.escribir(mensaje)`: Dibuja texto en la posición actual de la tortuga.

## Integración con Librería Gráfica

La tortuga funciona sobre la librería `graficos`. Puede usar funciones de `graficos` para control avanzado:

- `graficos.ventana(ancho, alto, titulo)`: Abre la ventana de dibujo.
- `graficos.limpiar(color)`: Borra la pantalla con un color de fondo.
- `graficos.actualizar()`: Refresca el contenido de la ventana. Es necesario llamar a esto dentro de los bucles para ver animaciones.

---

## Ejemplos Avanzados

### Espiral de Colores
```aguila
# Ver tortuga_espiral.ag
```

### Fractal (Árbol Recursivo)
```aguila
# Ver tortuga_arte.ag
```
