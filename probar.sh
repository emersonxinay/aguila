#!/bin/bash

# Colores
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🦅 Iniciando Suite de Pruebas de Águila..."

# 1. Pruebas Unitarias de Rust
echo -e "\n📦 Ejecutando pruebas unitarias (Cargo)..."
cd aguila
if cargo test --quiet; then
    echo -e "${GREEN}✔ Pruebas unitarias pasaron.${NC}"
else
    echo -e "${RED}✘ Fallaron las pruebas unitarias.${NC}"
    exit 1
fi

# 2. Compilar binario para pruebas de integración
echo -e "\n🔨 Compilando binario release..."
if cargo build --release --quiet; then
    echo -e "${GREEN}✔ Compilación exitosa.${NC}"
else
    echo -e "${RED}✘ Falló la compilación.${NC}"
    exit 1
fi

BIN="./target/release/aguila"

# 3. Pruebas de Integración (Scripts .ag)
echo -e "\n📜 Ejecutando scripts de prueba..."

run_script() {
    file=$1
    echo -n "  - Ejecutando $file... "
    if $BIN "$file" > /dev/null; then
        echo -e "${GREEN}OK${NC}"
    else
        echo -e "${RED}FALLÓ${NC}"
        exit 1
    fi
}

run_script "ejemplos/hola.ag"
run_script "ejemplos/hola.ag"
run_script "ejemplos/algoritmos_avanzados.ag"

# 4. Pruebas de Compatibilidad (Regresión)
echo -e "\n🏛️  Ejecutando pruebas de compatibilidad (v2.x)..."
# Estamos en 'aguila/', así que 'pruebas' está en '../pruebas'
COMPAT_DIR="../pruebas/compatibilidad"

if [ -d "$COMPAT_DIR" ]; then
    for test_file in "$COMPAT_DIR"/*.ag; do
        if [ -f "$test_file" ]; then
            echo -n "  - Ejecutando $test_file... "
            # El binario se ejecuta desde 'aguila/', así que la ruta relativa '../pruebas/...' es válida
            if $BIN "$test_file" > /dev/null; then
                echo -e "${GREEN}OK${NC}"
            else
                echo -e "${RED}FALLÓ${NC}"
                exit 1
            fi
        fi
    done
else
    echo "Advertencia: No se encontró directorio de compatibilidad."
fi

# Prueba con input (Gestor de Tareas - Opción 4: Salir)
echo -n "  - Ejecutando ejemplos/app_completa/gestor_tareas.ag... "
if echo "4" | $BIN "ejemplos/app_completa/gestor_tareas.ag" > /dev/null; then
    echo -e "${GREEN}OK${NC}"
else
    echo -e "${RED}FALLÓ${NC}"
    exit 1
fi

echo -e "\n✨ ¡Todas las pruebas pasaron exitosamente!"
cd ..
