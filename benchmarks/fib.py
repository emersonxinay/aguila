import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

inicio = time.time()
resultado = fib(40)
fin = time.time()

print(f"Python: Fib(40) = {resultado}")
print(f"Tiempo: {fin - inicio:.4f} segundos")
