#!/bin/bash

# Colores
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== Iniciando Suite de Benchmarks Águila ===${NC}"

# Asegurar que el compilador esté construido
echo "Construyendo Águila..."
cargo build --release --quiet

AGUILA="./target/release/aguila"

run_bench() {
    FILE=$1
    NAME=$2
    
    echo -e "\n${GREEN}>>> Benchmark: $NAME ($FILE)${NC}"
    
    # echo "--- Modo Intérprete (VM) ---"
    # $AGUILA vm $FILE
    
    echo -e "\n--- Modo AOT (Compilado) ---"
    # Compilar primero
    $AGUILA compilar $FILE
    
    # Ejecutar binario resultante (asumiendo que se genera sin extensión .ag)
    BIN="${FILE%.ag}"
    if [ -f "$BIN" ]; then
        # Medir tiempo de ejecución del binario
        time ./$BIN
        # Limpiar
        rm $BIN
        if [ -f "$BIN.c" ]; then rm "$BIN.c"; fi
    else
        echo -e "${RED}Error: No se generó el binario $BIN${NC}"
    fi
}

# 1. Fibonacci (CPU Recursivo)
# Usamos el existente si está, sino creamos uno temporal
if [ ! -f "benchmarks/fib.ag" ]; then
    echo "funcion fib(n) { si n < 2 { retornar n } retornar fib(n-1) + fib(n-2) } imprimir fib(30)" > benchmarks/fib.ag
fi
run_bench "benchmarks/fib.ag" "Fibonacci(30)"

# 2. Mandelbrot (Float + Loops)
run_bench "benchmarks/mandelbrot.ag" "Mandelbrot Set"

# 3. Strings (Memoria)
run_bench "benchmarks/strings.ag" "Estrés de Cadenas"

echo -e "\n${GREEN}=== Suite Completada ===${NC}"
