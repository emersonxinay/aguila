# Crear Release en GitHub para ÁGUILA v0.2.0

## 📋 Checklist Pre-Release

- [ ] Compilar binario en modo release
- [ ] Verificar que .gitignore excluye archivos pesados
- [ ] Hacer commit de cambios finales
- [ ] Crear tag de versión
- [ ] Push a GitHub
- [ ] Crear GitHub Release con binarios

## 🔨 Paso 1: Compilar Binario

```bash
cd aguila
cargo build --release
cd ..

# El binario estará en: aguila/target/release/aguila
# Tamaño aproximado: 15-20 MB
```

## 📝 Paso 2: Commit y Tag

```bash
# Ver estado
git status

# Agregar cambios
git add .

# Commit
git commit -m "feat: Release v0.2.0 - VS Code extension and standard library

- Added VS Code extension (published on Marketplace)
- Implemented 'mate' module with math functions
- Implemented 'fecha' module with date/time functions
- Added async/await syntax support
- Updated documentation and README"

# Crear tag
git tag -a v0.2.0 -m "Release v0.2.0: VS Code extension and async/await support"

# Push
git push origin main --tags
```

## 🚀 Paso 3: Crear GitHub Release

### Opción A: Desde la Web (Recomendado)

1. Ve a: https://github.com/emersonxinay/aguila/releases/new
2. Selecciona el tag: `v0.2.0`
3. Título: `v0.2.0 - VS Code Extension & Standard Library`
4. Descripción (copiar el texto de abajo)
5. Adjuntar binarios:
   - **macOS**: `aguila/target/release/aguila` (renombrar a `aguila-macos-v0.2.0`)
   - Comprimir: `zip aguila-macos-v0.2.0.zip aguila-macos-v0.2.0`
6. Click en "Publish release"

### Opción B: Desde GitHub CLI

```bash
# Instalar gh si no lo tienes
brew install gh

# Login
gh auth login

# Crear release
gh release create v0.2.0 \
  --title "v0.2.0 - VS Code Extension & Standard Library" \
  --notes-file RELEASE_NOTES.md \
  aguila/target/release/aguila#aguila-macos-v0.2.0
```

## 📄 Notas del Release (Copiar en GitHub)

```markdown
## 🎉 ÁGUILA v0.2.0

### 🆕 Nuevas Características

#### Extensión VS Code Oficial
- 🔌 **Publicada en Marketplace**: [Instalar extensión](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)
- Resaltado de sintaxis completo para archivos `.ag`
- Icono personalizado del águila
- Auto-cierre de brackets y paréntesis
- Soporte para comentarios con `//`

#### Módulo `mate` (Matemáticas)
```aguila
mate.pi              # 3.141592...
mate.sin(0)          # Seno
mate.cos(0)          # Coseno
mate.raiz(16)        # Raíz cuadrada
mate.potencia(2, 3)  # Potencia
mate.aleatorio()     # Número aleatorio
```

#### Módulo `fecha` (Fechas)
```aguila
t = fecha.ahora()
fecha.formato(t, "%Y-%m-%d %H:%M:%S")
```

#### Sintaxis Async/Await
```aguila
asincrono funcion obtener_datos() {
    respuesta = esperar fetch("https://api.com")
    retornar respuesta
}
```

### 🔧 Mejoras
- Mejor generación de código JavaScript
- Soporte completo para closures asíncronos
- Interpolación de cadenas mejorada
- Compilación optimizada en modo release

### 📦 Instalación

**Extensión VS Code:**
```bash
code --install-extension aguila-lang.aguila-vscode
```

**Binario (macOS):**
1. Descarga `aguila-macos-v0.2.0.zip`
2. Extrae y mueve a `/usr/local/bin/`
3. `chmod +x /usr/local/bin/aguila`

**Via npm (próximamente):**
```bash
npm install -g aguila-lang
```

### 📚 Recursos
- [Extensión VS Code](https://marketplace.visualstudio.com/items?itemName=aguila-lang.aguila-vscode)
- [Documentación](https://github.com/emersonxinay/aguila)
- [Ejemplos](https://github.com/emersonxinay/aguila/tree/main/ejemplos)

---
Hecho con ❤️ por Emerson Espinoza
```

## 📊 Tamaños de Descarga

| Componente | Tamaño | Usuario |
|------------|--------|---------|
| Extensión VS Code | ~300 KB | ✅ Todos |
| Binario macOS | ~15 MB | ✅ Usuarios finales |
| Repositorio completo | ~500 MB | ❌ Solo desarrolladores |

**Nota**: Los usuarios finales NO necesitan clonar el repositorio completo.

## ✅ Verificación Post-Release

- [ ] Release visible en GitHub
- [ ] Binario descargable
- [ ] Extensión actualizada en Marketplace
- [ ] README actualizado
- [ ] Links funcionando
