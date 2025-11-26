const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

// Configuración
const REPO = 'emersonxinay/agila';
const VERSION = 'v1.1.0'; // Debe coincidir con el tag de GitHub
const BIN_NAME = process.platform === 'win32' ? 'aguila.exe' : 'aguila';

// Detectar plataforma
function getAssetUrl() {
    const platform = process.platform;
    if (platform === 'win32') return `https://github.com/${REPO}/releases/download/${VERSION}/aguila-windows.exe`;
    if (platform === 'darwin') return `https://github.com/${REPO}/releases/download/${VERSION}/aguila-macos`;
    if (platform === 'linux') return `https://github.com/${REPO}/releases/download/${VERSION}/aguila-linux`;
    throw new Error(`Plataforma no soportada: ${platform}`);
}

const finalDest = path.join(__dirname, BIN_NAME);
const initialUrl = getAssetUrl();

console.log(`🦅 Instalando ÁGUILA para ${process.platform}...`);

function download(url, dest) {
    return new Promise((resolve, reject) => {
        const req = https.get(url, (res) => {
            if (res.statusCode === 301 || res.statusCode === 302) {
                // Seguir redirección
                download(res.headers.location, dest).then(resolve).catch(reject);
                return;
            }

            if (res.statusCode !== 200) {
                reject(new Error(`Falló la descarga con código: ${res.statusCode}`));
                return;
            }

            const file = fs.createWriteStream(dest);
            res.pipe(file);

            file.on('finish', () => {
                file.close();
                resolve();
            });

            file.on('error', (err) => {
                fs.unlink(dest, () => { });
                reject(err);
            });
        });

        req.on('error', (err) => {
            reject(err);
        });
    });
}

download(initialUrl, finalDest)
    .then(() => {
        if (process.platform !== 'win32') {
            execSync(`chmod +x ${finalDest}`);
        }
        console.log('✅ Instalación completada.');
    })
    .catch((err) => {
        console.error(`❌ Error: ${err.message}`);
        process.exit(1);
    });
