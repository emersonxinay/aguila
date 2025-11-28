# Guía de Lanzamiento: Águila v2.3.0

## 📋 Checklist Pre-Release

- [x] Compilar binario en modo release
- [x] Actualizar versión en `Cargo.toml` (2.3.0)
- [x] Actualizar versión en `npm/package.json` (2.3.0)
- [x] Actualizar versión en `aguila-vscode/package.json` (0.3.0)
- [ ] Crear tag de versión
- [ ] Generar binarios para todas las plataformas
- [ ] Crear GitHub Release
- [ ] Publicar en NPM
- [ ] Publicar extensión VS Code

---

## 🔨 Paso 1: Compilar Binarios

### macOS (Apple Silicon)
```bash
cd aguila
cargo build --release
cp target/release/aguila ../binarios/aguila-macos-arm64-v2.3.0
```

### macOS (Intel)
```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/aguila ../binarios/aguila-macos-x64-v2.3.0
```

### Linux (x86_64)
```bash
# En GitHub Actions o máquina Linux
cargo build --release
cp target/release/aguila ../binarios/aguila-linux-x64-v2.3.0
```

### Windows (x86_64)
```bash
# En GitHub Actions o máquina Windows
cargo build --release
copy target\release\aguila.exe ..\binarios\aguila-windows-x64-v2.3.0.exe
```

---

## 📝 Paso 2: Crear Tag y Commit

```bash
# Commit final
git add .
git commit -m "release: v2.3.0 - Asignación a índices, romper, métodos optimizados"

# Crear tag
git tag -a v2.3.0 -m "Release v2.3.0: Asignación a índices + métodos optimizados"

# Push
git push origin main --tags
```

---

## 🚀 Paso 3: Crear GitHub Release

### Título del Release
```
v2.3.0 - Asignación a Índices + Métodos Optimizados
```

### Descripción (Copiar en GitHub)

```markdown
## 🦅 Águila v2.3.0

### 🚀 Nuevas Características

#### 1️⃣ Asignación a Índices
¡La característica más esperada! Ahora puedes modificar listas y diccionarios directamente:

```aguila
# Listas
lista = [1, 2, 3, 4, 5]
lista[0] = 100
lista[4] = 500

# Diccionarios
config = {"puerto": 3000}
config["puerto"] = 8080
```

**Algoritmos desbloqueados:**
- ✅ N-Reinas
- ✅ Sudoku Solver
- ✅ Floyd-Warshall
- ✅ Knapsack (Mochila)
- ✅ Programación dinámica

#### 2️⃣ Palabra Clave `romper` (Break)

```aguila
mientras verdadero {
    x = leer("Número: ")
    si x == secreto {
        imprimir "¡Ganaste!"
        romper
    }
}
```

#### 3️⃣ Métodos Nativos Optimizados

```aguila
numeros = [5, 2, 8, 1, 9, 3]
total = numeros.suma()      # 28
menor = numeros.minimo()    # 1
mayor = numeros.maximo()    # 9
```

---

### 📊 Comparación con Python

**Python:**
```python
nums = [5, 2, 8, 1, 9]
print(sum(nums))
print(min(nums))
```

**Águila - MÁS SIMPLE:**
```aguila
numeros = [5, 2, 8, 1, 9]
imprimir numeros.suma()
imprimir numeros.minimo()
```

---

### 📦 Instalación

#### NPM (Recomendado)
```bash
npm install -g aguila-lang@2.3.0
```

#### Binarios Standalone
Descarga el binario para tu plataforma:
- **macOS (Apple Silicon):** `aguila-macos-arm64-v2.3.0`
- **macOS (Intel):** `aguila-macos-x64-v2.3.0`
- **Linux (x86_64):** `aguila-linux-x64-v2.3.0`
- **Windows (x86_64):** `aguila-windows-x64-v2.3.0.exe`

**Instalación en macOS/Linux:**
```bash
chmod +x aguila-macos-arm64-v2.3.0
sudo mv aguila-macos-arm64-v2.3.0 /usr/local/bin/aguila
aguila --version
```

#### VS Code Extension
```bash
code --install-extension aguila-lang.aguila-vscode
```

---

### 🎯 Ejemplos Nuevos

- **N-Reinas completo:** `aguila/ejemplos/n_reinas.ag`
- **Métodos nativos:** `aguila/ejemplos/test_metodos.ag`
- **Asignación a índices:** `aguila/ejemplos/test_asignacion_indice.ag`

---

### 🔧 Mejoras Técnicas

- **Performance:** Asignación O(1) a listas y diccionarios
- **Sintaxis:** Más concisa que Python
- **Optimización:** Métodos nativos con iteradores Rust

---

### 📚 Recursos

- [Tutorial Completo](TUTORIAL.md)
- [Comparación con Python](AGUILA_VS_PYTHON.md)
- [Documentación](DOCUMENTACION.md)
- [Plan de Optimización](plan_optimizacion.md)

---

**Hecho con ❤️ para la comunidad hispanohablante**

🦅 Águila - Programación en español, velocidad de Rust
```

### Adjuntar Binarios

1. Ve a: https://github.com/emersonxinay/aguila/releases/new
2. Selecciona el tag: `v2.3.0`
3. Título: `v2.3.0 - Asignación a Índices + Métodos Optimizados`
4. Descripción: Pegar el texto de arriba
5. Adjuntar binarios:
   - `aguila-macos-arm64-v2.3.0`
   - `aguila-macos-x64-v2.3.0`
   - `aguila-linux-x64-v2.3.0`
   - `aguila-windows-x64-v2.3.0.exe`
6. Click en "Publish release"

---

## 📦 Paso 4: Publicar en NPM

```bash
cd npm
npm publish
```

**Verificar:**
```bash
npm info aguila-lang
```

---

## 🎨 Paso 5: Publicar Extensión VS Code

```bash
cd aguila-vscode
vsce package
vsce publish
```

**Verificar:**
https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode

---

## ✅ Paso 6: Verificación Post-Release

- [ ] Release visible en GitHub
- [ ] Binarios descargables
- [ ] NPM package actualizado
- [ ] Extensión VS Code actualizada
- [ ] README actualizado
- [ ] Links funcionando

---

## 🎉 Paso 7: Anuncio

### Twitter/X
```
🦅 Águila v2.3.0 ya está disponible!

✨ Asignación a índices (lista[i] = valor)
✨ Palabra clave 'romper' (break)
✨ Métodos .suma(), .minimo(), .maximo()

Ahora más simple que Python 🐍

npm install -g aguila-lang@2.3.0

#Aguila #ProgramaciónEnEspañol
```

### Reddit (r/ProgrammingLanguages)
```
Águila v2.3.0 Released - Spanish Programming Language

New features:
- Index assignment (list[i] = value)
- Break keyword ('romper')
- Optimized methods (.suma(), .minimo(), .maximo())

Now simpler than Python with Rust performance.

GitHub: https://github.com/emersonxinay/aguila
```

---

**¡Listo para lanzar! 🚀**
