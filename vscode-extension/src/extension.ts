import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    const serverModule = context.asAbsolutePath(
        path.join('..', 'target', 'release', 'lsp')
    );

    const serverOptions: ServerOptions = {
        run: { command: serverModule, transport: TransportKind.stdio },
        debug: { command: serverModule, transport: TransportKind.stdio }
    };

    const clientOptions: LanguageClientOptions = {
        // Document selector tells VS Code to watch files with the .psy extension
        documentSelector: [{ scheme: 'file', language: 'psy' }],
        synchronize: {
            // Synchronize configuration sections to the server context loop
            configurationSection: 'psyServer',
            fileEvents: workspace.createFileSystemWatcher('**/*.psy')
        }
    };

    client = new LanguageClient(
        'psyLanguageServer',
        'Psy Language Server',
        serverOptions,
        clientOptions
    );

    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}