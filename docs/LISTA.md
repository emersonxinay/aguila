# Documentación del Módulo `lista`

El módulo `lista` proporciona funciones nativas para la manipulación eficiente de listas (arrays dinámicos) en Águila.

## Funciones

### `lista.longitud(lst: Lista) -> Entero`
Retorna el número de elementos en la lista.
- También funciona con cadenas de texto.

### `lista.agregar(lst: Lista, elemento: Cualquiera)`
Añade un elemento al final de la lista. Modifica la lista original (in-place).

### `lista.insertar(lst: Lista, indice: Entero, elemento: Cualquiera)`
Inserta un elemento en una posición específica, desplazando los elementos posteriores.
- **indice**: Posición base-0.

### `lista.eliminar(lst: Lista, indice: Entero) -> Cualquiera`
Elimina el elemento en el índice dado y lo retorna. Desplaza los elementos posteriores para llenar el hueco.

### `lista.limpiar(lst: Lista)`
Elimina todos los elementos de la lista, dejándola vacía.

### `lista.invertir(lst: Lista)`
Invierte el orden de los elementos de la lista in-place.

### `lista.unir(lst: Lista, separador: Texto) -> Texto`
Convierte todos los elementos de la lista a texto y los une en una sola cadena, separados por el `separador`.

## Ejemplo

```aguila
importar lista

mi_lista = []
lista.agregar(mi_lista, 10)         # [10]
lista.insertar(mi_lista, 0, 5)      # [5, 10]
lista.agregar(mi_lista, 20)         # [5, 10, 20]

removido = lista.eliminar(mi_lista, 1) # quita 10
imprimir(removido) # 10
imprimir(mi_lista) # [5, 20]
```
