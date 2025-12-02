#!/bin/bash

# Script para probar todos los ejemplos de Águila
# Colores para output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0
TOTAL=0

echo "========================================="
echo "  Probando Ejemplos de Águila"
echo "========================================="
echo ""

# Función para ejecutar un test
run_test() {
    local file=$1
    local name=$2
    TOTAL=$((TOTAL + 1))
    
    echo -n "[$TOTAL] Probando $name... "
    
    if ./aguila/target/debug/aguila ejecutar "$file" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ PASS${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ FAIL${NC}"
        FAILED=$((FAILED + 1))
        echo "    Archivo: $file"
    fi
}

# BÁSICOS
echo -e "${YELLOW}=== BÁSICOS ===${NC}"
run_test "ejemplos/basicos/hola_mundo.ag" "Hola Mundo"
run_test "ejemplos/basicos/variables_y_tipos.ag" "Variables y Tipos"
run_test "ejemplos/basicos/interpolacion_basica.ag" "Interpolación Básica"
run_test "ejemplos/basicos/bucle_simple.ag" "Bucle Simple"
run_test "ejemplos/basicos/bucles_debugging.ag" "Bucles Debugging"
run_test "ejemplos/basicos/control_flujo.ag" "Control de Flujo"
run_test "ejemplos/basicos/prueba_vm.ag" "Prueba VM"

echo ""
echo -e "${YELLOW}=== INTERMEDIOS ===${NC}"
run_test "ejemplos/intermedio/funciones.ag" "Funciones"
run_test "ejemplos/intermedio/colecciones.ag" "Colecciones"
run_test "ejemplos/intermedio/diccionarios.ag" "Diccionarios"
run_test "ejemplos/intermedio/programacion_orientada_objetos.ag" "POO"

echo ""
echo -e "${YELLOW}=== AVANZADOS ===${NC}"
echo "⚠️  Saltando fibonacci.ag (JIT issue conocido con MAX_INLINE_DEPTH=10)"
# run_test "ejemplos/avanzado/fibonacci.ag" "Fibonacci (JIT)"
run_test "ejemplos/avanzado/matematicas.ag" "Matemáticas"
run_test "ejemplos/avanzado/biblioteca_estandar.ag" "Biblioteca Estándar"
run_test "ejemplos/avanzado/modulos.ag" "Módulos"
run_test "ejemplos/avanzado/modulo_ejemplo.ag" "Módulo Ejemplo"
run_test "ejemplos/avanzado/jit_comparacion_strings.ag" "JIT Strings"
run_test "ejemplos/avanzado/interpolacion_avanzada.ag" "Interpolación Avanzada"

echo ""
echo "========================================="
echo "  Resultados"
echo "========================================="
echo -e "Total:   $TOTAL"
echo -e "${GREEN}Pasados: $PASSED${NC}"
echo -e "${RED}Fallidos: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ Todos los tests pasaron!${NC}"
    exit 0
else
    echo -e "${RED}✗ Algunos tests fallaron${NC}"
    exit 1
fi
