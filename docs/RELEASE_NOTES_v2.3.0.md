#  Águila v2.3.0 - Release Notes

##  Nuevas Características

### 1. Asignación a Índices
**¡La característica más esperada!** Ahora puedes modificar elementos de listas y diccionarios directamente:

```aguila
# Listas
lista = [1, 2, 3, 4, 5]
lista[0] = 100
lista[4] = 500
imprimir lista  # [100, 2, 3, 4, 500]

# Diccionarios
config = {"puerto": 3000, "host": "localhost"}
config["puerto"] = 8080
config["ssl"] = verdadero
imprimir config
```

**Algoritmos desbloqueados:**
- ✅ N-Reinas
- ✅ Sudoku Solver
- ✅ Floyd-Warshall
- ✅ Knapsack (Mochila)
- ✅ Multiplicación de matrices
- ✅ Programación dinámica

### 2. Palabra Clave `romper`
Control de flujo mejorado con `romper` (break):

```aguila
# Búsqueda con salida temprana
para i = 0 hasta 100 {
    si lista[i] == objetivo {
        imprimir "¡Encontrado!"
        romper
    }
}

# Juego de adivinanza
secreto = 7
mientras verdadero {
    x = leer("Número: ")
    si x == secreto {
        imprimir "¡Ganaste!"
        romper
    }
}
```

### 3. Métodos Nativos Optimizados
Nuevos métodos para listas con implementación O(n) en Rust:

```aguila
numeros = [5, 2, 8, 1, 9, 3]

total = numeros.suma()      # 28
menor = numeros.minimo()    # 1
mayor = numeros.maximo()    # 9

# Calcular promedio
promedio = numeros.suma() / numeros.longitud()
imprimir promedio  # 4.67
```

---

##  Comparación con Python

### Antes (v2.2.8)
```aguila
# ❌ No funcionaba
lista[0] = 100  # Error

# ❌ No había break
mientras verdadero {
    # Sin forma de salir
}

# ❌ Sin métodos agregados
suma = 0
para n en lista {
    suma += n
}
```

### Ahora (v2.3.0)
```aguila
# ✅ Asignación directa
lista[0] = 100

# ✅ Break nativo
mientras verdadero {
    si condicion {
        romper
    }
}

# ✅ Métodos optimizados
suma = lista.suma()
```

### vs Python
```python
# Python
nums = [5, 2, 8, 1, 9]
print(sum(nums))
print(min(nums))
print(max(nums))
```

```aguila
# Águila - MÁS SIMPLE
numeros = [5, 2, 8, 1, 9]
imprimir numeros.suma()
imprimir numeros.minimo()
imprimir numeros.maximo()
```

---

##  Ejemplos Nuevos

### N-Reinas Completo
```aguila
funcion n_reinas(n) {
    tablero = []
    para i = 0 hasta n {
        tablero.agregar(-1)
    }
    
    soluciones = []
    resolver(0, tablero, soluciones, n)
    retornar soluciones
}

funcion resolver(fila, tablero, soluciones, n) {
    si fila == n {
        soluciones.agregar(tablero.copiar())
        retornar
    }

    para col = 0 hasta n {
        si valido(tablero, fila, col) {
            tablero[fila] = col  # ✅ Ahora funciona!
            resolver(fila + 1, tablero, soluciones, n)
            tablero[fila] = -1
        }
    }
}

# Uso
soluciones = n_reinas(8)
imprimir a"Encontradas {soluciones.longitud()} soluciones"
```

Ver ejemplo completo en: `aguila/ejemplos/n_reinas.ag`

---

##  Mejoras Técnicas

### Performance
- **Asignación a listas:** O(1) - acceso directo al vector
- **Asignación a diccionarios:** O(1) amortizado - HashMap de Rust
- **Métodos agregados:** O(n) con iteradores optimizados

### Sintaxis
- ✅ Más concisa que Python en muchos casos
- ✅ Inferencia de tipos automática
- ✅ Sin conversiones manuales (`int()`, `str()`)

---

##  Instalación

### NPM (Actualizar)
```bash
npm install -g aguila-lang@2.3.0
```

### Desde código fuente
```bash
git clone https://github.com/emersonxinay/aguila.git
cd aguila/aguila
cargo build --release
```

### VS Code Extension
```bash
code --install-extension aguila-lang.aguila-vscode
```

---

##  Correcciones

- Corregido: Parser ahora detecta asignación a índices correctamente
- Corregido: `romper` funciona en bucles anidados
- Mejorado: Mensajes de error más claros para índices fuera de rango

---

##  Limitaciones Conocidas

- Asignación a índices anidados (`matriz[i][j] = valor`) requiere workaround temporal
- No hay `continuar` (continue) aún - próxima versión

---

##  Recursos

- [Tutorial Completo](TUTORIAL.md)
- [Comparación con Python](AGUILA_VS_PYTHON.md)
- [Documentación](DOCUMENTACION.md)
- [Ejemplos](aguila/ejemplos/)

---

##  Agradecimientos

Gracias a la comunidad por el feedback y sugerencias. Esta versión implementa las características más solicitadas.

---

**Hecho con ❤️ para la comunidad hispanohablante**

 Águila - Programación en español, velocidad de Rust
