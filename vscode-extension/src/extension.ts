import * as path from "path";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate() {
    const serverPath = path.join(
        __dirname,
        "..",
        "..",
        "target",
        "release",
        "psy-lsp"
    );

    const serverOptions: ServerOptions = {
        run: {
            command: serverPath,
            transport: TransportKind.stdio,
            args: []
        },
        debug: {
            command: serverPath,
            transport: TransportKind.stdio,
            args: ["--debug"]
        },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [
            { scheme: "file", language: "psy" }
        ],
        synchronize: {
            fileEvents: []
        },
        // Initialize options 
        initializationOptions: {
            hoverEnabled: true,
            diagnosticsEnabled: true
        }
    };

    client = new LanguageClient(
        "psyLsp",
        "Psy Language Server",
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