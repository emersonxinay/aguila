import * as vscode from 'vscode';
import * as https from 'https';
import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

export function activate(context: vscode.ExtensionContext) {
    console.log('Águila extension is now active!');

    // Verificar si Águila está instalado
    if (!isAguilaInstalled()) {
        vscode.window.showInformationMessage(
            'Águila no está instalado. ¿Deseas instalarlo automáticamente?',
            'Sí, instalar', 'No'
        ).then(selection => {
            if (selection === 'Sí, instalar') {
                installAguila(context);
            }
        });
    }
}

export function deactivate() { }

function isAguilaInstalled(): boolean {
    try {
        execSync('aguila --version', { stdio: 'ignore' });
        return true;
    } catch {
        return false;
    }
}

async function installAguila(context: vscode.ExtensionContext) {
    const platform = process.platform;
    const arch = process.arch;

    // URL del binario según plataforma (v2.3.0)
    let downloadUrl = '';
    let binaryName = 'aguila';

    if (platform === 'darwin') {
        if (arch === 'arm64') {
            downloadUrl = 'https://github.com/emersonxinay/aguila/releases/download/v2.3.0/aguila-macos';
        } else {
            downloadUrl = 'https://github.com/emersonxinay/aguila/releases/download/v2.3.0/aguila-macos'; // Fallback to same binary or x64 if available
        }
    } else if (platform === 'linux') {
        downloadUrl = 'https://github.com/emersonxinay/aguila/releases/download/v2.3.0/aguila-linux';
    } else if (platform === 'win32') {
        downloadUrl = 'https://github.com/emersonxinay/aguila/releases/download/v2.3.0/aguila-windows.exe';
        binaryName = 'aguila.exe';
    } else {
        vscode.window.showErrorMessage(`Plataforma no soportada automáticamente: ${platform}-${arch}`);
        return;
    }

    vscode.window.withProgress({
        location: vscode.ProgressLocation.Notification,
        title: "Instalando Águila...",
        cancellable: false
    }, async (progress) => {
        try {
            // Crear carpeta bin si no existe
            const binDir = path.join(context.globalStorageUri.fsPath, 'bin');
            if (!fs.existsSync(binDir)) {
                fs.mkdirSync(binDir, { recursive: true });
            }

            const binPath = path.join(binDir, binaryName);

            progress.report({ message: "Descargando..." });
            await downloadFile(downloadUrl, binPath);

            // Dar permisos de ejecución (macOS/Linux)
            if (platform !== 'win32') {
                progress.report({ message: "Configurando permisos..." });
                fs.chmodSync(binPath, '755');
            }

            // Agregar al PATH (esto es complejo de hacer persistentemente desde la extensión, 
            // así que mejor configuramos la extensión para usar este path o instruimos al usuario)

            // Opción A: Configurar la extensión para usar este ejecutable
            const config = vscode.workspace.getConfiguration('aguila');
            await config.update('executablePath', binPath, vscode.ConfigurationTarget.Global);

            vscode.window.showInformationMessage(`¡Águila instalado correctamente en ${binPath}! Se ha configurado la extensión para usarlo.`);

        } catch (error) {
            vscode.window.showErrorMessage(`Error instalando Águila: ${error}`);
        }
    });
}

function downloadFile(url: string, dest: string): Promise<void> {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(dest);
        https.get(url, (response) => {
            if (response.statusCode !== 200) {
                reject(new Error(`Falló la descarga: ${response.statusCode}`));
                return;
            }
            response.pipe(file);
            file.on('finish', () => {
                file.close();
                resolve();
            });
        }).on('error', (err) => {
            fs.unlink(dest, () => { });
            reject(err);
        });
    });
}
