use psy_checker::{Diagnostic as PscDiagnostic, Severity as PscSeverity};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "psy-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "psy-lsp initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.check_and_publish(params.text_document.uri, &params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // We registered FULL sync, so the last change content_change
        // entry contains the entire document text.
        if let Some(change) = params.content_changes.into_iter().last() {
            self.check_and_publish(params.text_document.uri, &change.text)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // Clear diagnostics when a file closes, so stale squiggles don't
        // linger in the editor's problems panel.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }
}

impl Backend {
    async fn check_and_publish(&self, uri: Url, source: &str) {
        let psc_diagnostics = psy_checker::check(source);
        let lsp_diagnostics: Vec<Diagnostic> = psc_diagnostics
            .iter()
            .map(|d| convert_diagnostic(d))
            .collect();

        self.client
            .publish_diagnostics(uri, lsp_diagnostics, None)
            .await;
    }
}

/// Converts one of our checker's Diagnostic structs into an LSP
/// Diagnostic. Our line/column are 1-indexed (matching how editors and
/// compilers traditionally report positions to humans); LSP positions
/// are 0-indexed, so both need to be adjusted down by one.
fn convert_diagnostic(d: &PscDiagnostic) -> Diagnostic {
    let line = d.line.saturating_sub(1) as u32;
    let column = d.column.saturating_sub(1) as u32;

    let severity = match d.severity {
        PscSeverity::Error => DiagnosticSeverity::ERROR,
        PscSeverity::Warning => DiagnosticSeverity::WARNING,
    };

    let mut message = d.message.clone();
    if let Some(suggestion) = &d.suggestion {
        message.push_str("\nSuggestion: ");
        message.push_str(suggestion);
    }

    Diagnostic {
        range: Range {
            start: Position {
                line,
                character: column,
            },
            end: Position {
                line,
                character: column + 1,
            },
        },
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("psy-checker".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
