import * as path from "path";
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate() {
    // Path to the compiled psy-lsp binary
    const serverPath = path.join(
        __dirname,
        "..",
        "..",
        "target",
        "release",
        "psy-lsp"
    );

    const serverOptions: ServerOptions = {
        run: { command: serverPath, transport: TransportKind.stdio },
        debug: { command: serverPath, transport: TransportKind.stdio },
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: "file", language: "psy" }],
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