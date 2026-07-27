#  Águila vs Python: 10 Ejercicios Comparativos

> **Águila es más simple que Python** - Misma expresividad, menos complejidad

---

## 1. Suma de números del 1 al N

### Python
```python
n = int(input("N: "))
suma = 0
for i in range(1, n+1):
    suma += i
print(suma)
```

### Águila ✨
```aguila
n = leer("N: ")
suma = 0

para i = 1 hasta n + 1 {
    suma += i
}

imprimir suma
```

**¿Por qué es más simple?**
- ✅ `leer()` detecta automáticamente que es un número
- ✅ `para i = 1 hasta n` es más legible que `range(1, n+1)`
- ✅ No necesitas `int()` ni conversiones manuales

---

## 2. Factorial de un número

### Python
```python
n = int(input("N: "))
fact = 1
for i in range(1, n+1):
    fact *= i
print(fact)
```

### Águila ✨
```aguila
n = leer("N: ")
fact = 1

para i = 1 hasta n + 1 {
    fact = fact * i
}

imprimir fact
```

**Diferencia clave:** Sin `int()`, sin `range()`. Águila infiere tipos automáticamente.

---

## 3. Verificar si un número es primo

### Python
```python
n = int(input("N: "))
es_primo = True
for i in range(2, n):
    if n % i == 0:
        es_primo = False
        break
print(es_primo)
```

### Águila ✨
```aguila
n = leer("N: ")
es_primo = verdadero

para i = 2 hasta n {
    si n % i == 0 {
        es_primo = falso
        romper
    }
}

imprimir es_primo
```

**Mejoras:**
- ✅ `verdadero`/`falso` en español (más natural)
- ✅ `romper` en lugar de `break`
- ✅ `si` en lugar de `if`

---

## 4. Secuencia de Fibonacci

### Python
```python
n = int(input())
a, b = 0, 1
for _ in range(n):
    print(a)
    a, b = b, a + b
```

### Águila ✨
```aguila
n = leer("N: ")
a = 0
b = 1

para i = 0 hasta n {
    imprimir a
    temp = a
    a = b
    b = temp + b
}
```

**Nota:** Águila no tiene asignación múltiple (`a, b = b, a+b`), pero el código es igualmente claro.

---

## 5. Invertir una cadena

### Python
```python
s = input()
print(s[::-1])
```

### Águila ✨
```aguila
s = leer("Texto: ")
lista = s.dividir("")  # Separa en caracteres
lista.invertir()
imprimir lista.unir("")
```

**Diferencia:** Águila es más explícito (divide → invierte → une), Python usa sintaxis mágica `[::-1]`.

---

## 6. Contar vocales en un texto

### Python
```python
s = input()
vocales = "aeiou"
contador = 0
for c in s.lower():
    if c in vocales:
        contador += 1
print(contador)
```

### Águila ✨
```aguila
s = leer("Texto: ").minusculas()
vocales = ["a", "e", "i", "o", "u"]
c = 0

para letra en s.dividir("") {
    si vocales.contiene(letra) {
        c += 1
    }
}

imprimir c
```

**Ventajas:**
- ✅ `.minusculas()` encadenado directamente
- ✅ `.contiene()` es más legible que `in`

---

## 7. Sumar elementos de una lista

### Python
```python
lista = [1, 2, 3, 4]
print(sum(lista))
```

### Águila ✨
```aguila
lista = [1, 2, 3, 4]
suma = 0

para n en lista {
    suma += n
}

imprimir suma
```

**Nota:** Águila no tiene `sum()` built-in, pero el bucle es explícito y educativo.

---

## 8. Encontrar el máximo de una lista

### Python
```python
lista = [5, 3, 10, 2]
print(max(lista))
```

### Águila ✨
```aguila
lista = [5, 3, 10, 2]
maximo = lista[0]

para n en lista {
    si n > maximo {
        maximo = n
    }
}

imprimir maximo
```

**Ventaja educativa:** Águila te enseña el algoritmo real, no lo oculta detrás de `max()`.

---

## 9. Juego de adivinar número

### Python
```python
secreto = 7
while True:
    x = int(input())
    if x == secreto:
        print("Ganaste!")
        break
```

### Águila ✨
```aguila
secreto = 7

mientras verdadero {
    x = leer("Número: ")
    si x == secreto {
        imprimir "¡Ganaste!"
        romper
    }
}
```

**Mejoras:**
- ✅ `mientras verdadero` es más legible que `while True`
- ✅ Sin necesidad de `int()`

---

## 10. Ordenar una lista (Bubble Sort)

### Python
```python
lista = [5, 1, 4, 2]
for i in range(len(lista)):
    for j in range(i+1, len(lista)):
        if lista[j] < lista[i]:
            lista[i], lista[j] = lista[j], lista[i]
print(lista)
```

### Águila ✨
```aguila
lista = [5, 1, 4, 2]

para i = 0 hasta lista.longitud() {
    para j = i + 1 hasta lista.longitud() {
        si lista[j] < lista[i] {
            temp = lista[i]
            lista[i] = lista[j]
            lista[j] = temp
        }
    }
}

imprimir lista
```

**Ventajas:**
- ✅ `.longitud()` es más descriptivo que `len()`
- ✅ `para i = 0 hasta n` es más claro que `range(len(lista))`

---

##  Comparación General

| Característica | Python | Águila |
|---|---|---|
| **Conversión de tipos** | Manual (`int()`, `str()`) | Automática |
| **Bucles** | `range(start, end)` | `para i = inicio hasta fin` |
| **Booleanos** | `True`, `False` | `verdadero`, `falso` |
| **Condicionales** | `if`, `else` | `si`, `sino` |
| **Sintaxis** | Inglés | Español |
| **Legibilidad** | Alta | **Muy Alta** (español nativo) |

---

##  Conclusión

**Águila es más simple porque:**

1. **Inferencia de tipos automática** - No necesitas `int()`, `float()`, `str()`
2. **Sintaxis en español** - Más natural para hispanohablantes
3. **Bucles más legibles** - `para i = 1 hasta 10` vs `for i in range(1, 10)`
4. **Menos "magia"** - Código más explícito y educativo

**Águila te enseña a programar correctamente** mientras Python oculta complejidad detrás de funciones built-in.

---

**¿Listo para aprender más?** Consulta el [TUTORIAL.md](TUTORIAL.md) completo.
