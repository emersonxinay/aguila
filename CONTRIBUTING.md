# Guía de Contribución para ÁGUILA

¡Gracias por tu interés en contribuir a Águila!

## 🛠️ Configuración del Entorno

1.  **Instalar Rust:**
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

2.  **Clonar el repositorio:**
    ```bash
    git clone https://github.com/emersonxinay/aguila.git
    cd aguila
    ```

3.  **Compilar y Probar:**
    ```bash
    # Ejecutar todas las pruebas (unitarias + integración)
    ./probar.sh
    ```

## 🛡️ Política de Compatibilidad

Nos tomamos muy en serio la estabilidad. Seguimos **Versionado Semántico (SemVer)**:

1.  **Compatibilidad hacia atrás:** El código escrito para Águila v2.0 debe funcionar en v2.1, v2.2, etc.
2.  **Pruebas de Regresión:** Antes de cada PR, ejecutamos `pruebas/compatibilidad/`. Estos archivos son sagrados: si tu cambio rompe uno de estos tests, tu cambio será rechazado (a menos que sea una corrección de seguridad crítica o un cambio Major v3.0).

## 🧪 Cómo probar tus cambios

## 🚀 Estructura del Proyecto

- `aguila/`: Código fuente del compilador/intérprete (Rust).
- `aguila-vscode/`: Extensión para Visual Studio Code.
- `npm/`: Instalador para Node.js.
- `ejemplos/`: Scripts de ejemplo `.ag`.

## 📦 Proceso de Release

1.  **Actualizar Versión:**
    - `aguila/Cargo.toml`
    - `npm/package.json`
    - `aguila-vscode/package.json`

2.  **Generar Binarios:**
    GitHub Actions generará automáticamente los binarios para Linux, macOS y Windows al crear un nuevo Release.

3.  **Publicar:**
    - Crear Tag y Release en GitHub.
    - Publicar en NPM: `cd npm && npm publish`
    - Publicar en VS Code Marketplace: `cd aguila-vscode && vsce publish`

## 🤝 Normas de Código

- Usa `cargo fmt` antes de hacer commit.
- Asegúrate de que `./probar.sh` pase exitosamente.
- Documenta las nuevas funciones en `DOCUMENTACION.md`.
