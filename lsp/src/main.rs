use once_cell::sync::Lazy;
use psy_checker::symbols::{Symbol, SymbolKind};
use psy_checker::{Diagnostic as PsyDiagnostic, Severity as PsySeverity};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug, Deserialize)]
struct SymbolInfo {
    description: String,
    detail: String,
    example: String,
}

#[derive(Debug, Deserialize)]
struct ModuleInfo {
    description: String,
    detail: String,
    functions: HashMap<String, SymbolInfo>,
}

#[derive(Debug, Deserialize)]
struct Definitions {
    keywords: HashMap<String, SymbolInfo>,
    modules: HashMap<String, ModuleInfo>,
    builtins: HashMap<String, SymbolInfo>,
}

/// Per-document state tracked across open/change events.
#[derive(Default)]
struct DocumentState {
    content: String,
    /// Symbols collected from the parsed AST — variables, functions,
    /// consts, statics, imported names, etc.
    symbols: Vec<Symbol>,
    /// Set of function/identifier names actually referenced in calls,
    /// used to detect unused imports. Derived from the symbol list
    /// (Import-kind symbols that don't appear as Identifiers in the AST).
    imported_names: HashMap<String, String>, // name -> module
    used_names: std::collections::HashSet<String>,
}

static DEFINITIONS: Lazy<Definitions> = Lazy::new(|| {
    let paths_to_try = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("definitions.json"),
        PathBuf::from("lsp/definitions.json"),
        PathBuf::from("definitions.json"),
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|p| p.join("definitions.json")))
            .unwrap_or(PathBuf::from("definitions.json")),
        PathBuf::from("../lsp/definitions.json"),
    ];

    for path in paths_to_try.iter() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(defs) = serde_json::from_str(&content) {
                return defs;
            }
        }
    }

    panic!(
        "Failed to read definitions.json from any of the tried paths: {:?}",
        paths_to_try
    );
});

pub struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["_".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
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
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;

        let state = analyze_document(&text);
        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), state);
        }

        self.check_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            let uri = params.text_document.uri.clone();
            let text = change.text;

            let state = analyze_document(&text);
            {
                let mut docs = self.documents.lock().await;
                docs.insert(uri.clone(), state);
            }

            self.check_and_publish(&uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        {
            let mut docs = self.documents.lock().await;
            docs.remove(&params.text_document.uri);
        }

        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let position = params.text_document_position_params.position;
        let uri = params.text_document_position_params.text_document.uri;

        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let word_info = get_word_at_position(&doc_state.content, &position);
        let (word, range) = match word_info {
            Some(w) => w,
            None => return Ok(None),
        };

        // 1. Check user-defined symbols first (highest priority —
        //    user's own definitions should win over built-in docs)
        if let Some(hover_text) = hover_for_user_symbol(&word, &doc_state.symbols) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: Some(range),
            }));
        }

        // 2. Check static definitions (keywords, builtins, module fns)
        if let Some(info) = get_static_symbol_info(&word) {
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: info,
                }),
                range: Some(range),
            }));
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let _position = params.text_document_position.position;

        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut items: Vec<CompletionItem> = Vec::new();

        // Add keyword completions
        for (keyword, info) in &DEFINITIONS.keywords {
            items.push(CompletionItem {
                label: keyword.clone(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(info.description.clone()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("{}\n\n```psy\n{}\n```", info.detail, info.example),
                })),
                ..Default::default()
            });
        }

        // Add builtin completions
        for (name, info) in &DEFINITIONS.builtins {
            items.push(CompletionItem {
                label: name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(info.description.clone()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("{}\n\n```psy\n{}\n```", info.detail, info.example),
                })),
                ..Default::default()
            });
        }

        // Add imported module function completions
        for (module_name, module) in &DEFINITIONS.modules {
            for (func_name, func_info) in &module.functions {
                // Only offer if this module has been imported
                if doc_state.imported_names.contains_key(func_name) {
                    items.push(CompletionItem {
                        label: func_name.clone(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(format!("{} (from {})", func_info.description, module_name)),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!(
                                "{}\n\n```psy\n{}\n```",
                                func_info.detail, func_info.example
                            ),
                        })),
                        ..Default::default()
                    });
                }
            }
        }

        // Add user-defined symbols from the current document
        for symbol in &doc_state.symbols {
            let (kind, detail) = match &symbol.kind {
                SymbolKind::Function { parameters } => (
                    CompletionItemKind::FUNCTION,
                    format!("Function({})", parameters.join(", ")),
                ),
                SymbolKind::Variable => (CompletionItemKind::VARIABLE, "Variable".to_string()),
                SymbolKind::Const => (CompletionItemKind::CONSTANT, "Constant".to_string()),
                SymbolKind::Static => (CompletionItemKind::VARIABLE, "Static variable".to_string()),
                SymbolKind::Array { size } => {
                    (CompletionItemKind::VARIABLE, format!("Array[{}]", size))
                }
                SymbolKind::Import { module } => (
                    CompletionItemKind::FUNCTION,
                    format!("Imported from {}", module),
                ),
            };

            items.push(CompletionItem {
                label: symbol.name.clone(),
                kind: Some(kind),
                detail: Some(detail),
                insert_text: Some(match &symbol.kind {
                    SymbolKind::Function { parameters } if !parameters.is_empty() => {
                        // Snippet with placeholders for each parameter
                        let snippet_params: Vec<String> = parameters
                            .iter()
                            .enumerate()
                            .map(|(i, p)| format!("${{{}:{}}}", i + 1, p))
                            .collect();
                        format!("{}({})", symbol.name, snippet_params.join(", "))
                    }
                    SymbolKind::Function { .. } => format!("{}()", symbol.name),
                    _ => symbol.name.clone(),
                }),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        // Deduplicate by label (user symbols override static ones with same name)
        let mut seen = std::collections::HashSet::new();
        items.retain(|item| seen.insert(item.label.clone()));

        Ok(Some(CompletionResponse::Array(items)))
    }
}

impl Backend {
    async fn check_and_publish(&self, uri: &Url, source: &str) {
        let psy_diagnostics = psy_checker::check(source);
        let mut lsp_diagnostics: Vec<Diagnostic> = psy_diagnostics
            .iter()
            .map(|d| convert_diagnostic(d))
            .collect();

        // Add unused import warnings using AST-backed symbol info
        let docs = self.documents.lock().await;
        if let Some(state) = docs.get(uri) {
            for symbol in &state.symbols {
                if let SymbolKind::Import { module } = &symbol.kind {
                    if !state.used_names.contains(&symbol.name) {
                        lsp_diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: symbol.line.saturating_sub(1) as u32,
                                    character: symbol.column.saturating_sub(1) as u32,
                                },
                                end: Position {
                                    line: symbol.line.saturating_sub(1) as u32,
                                    character: (symbol.column.saturating_sub(1) + symbol.name.len())
                                        as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: None,
                            code_description: None,
                            source: Some("psy-lsp".to_string()),
                            message: format!(
                                "Imported '{}' from {} is never used",
                                symbol.name, module
                            ),
                            related_information: None,
                            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                            data: None,
                        });
                    }
                }
            }
        }

        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }
}

/// Analyzes a document by running the real parser and collecting symbols
/// from the AST — no regex, no fragile text scanning.
fn analyze_document(content: &str) -> DocumentState {
    let symbols = psy_checker::symbols(content);

    let mut imported_names: HashMap<String, String> = HashMap::new();
    let mut used_names = std::collections::HashSet::new();

    for symbol in &symbols {
        match &symbol.kind {
            SymbolKind::Import { module } => {
                imported_names.insert(symbol.name.clone(), module.clone());
            }
            SymbolKind::Variable | SymbolKind::Const | SymbolKind::Static => {
                used_names.insert(symbol.name.clone());
            }
            _ => {}
        }
    }

    DocumentState {
        content: content.to_string(),
        symbols,
        imported_names,
        used_names,
    }
}

/// Generates hover text for a user-defined symbol found in the document's
/// symbol table — functions show their signature, variables/consts show
/// their kind and declaration line.
fn hover_for_user_symbol(word: &str, symbols: &[Symbol]) -> Option<String> {
    let symbol = symbols.iter().find(|s| s.name == word)?;

    match &symbol.kind {
        SymbolKind::Function { parameters } => {
            let sig = if parameters.is_empty() {
                format!("FUNCTION {}()", symbol.name)
            } else {
                format!("FUNCTION {}({})", symbol.name, parameters.join(", "))
            };
            Some(format!(
                "```psy\n{}\n```\n\n*User-defined function* — declared at line {}",
                sig, symbol.line
            ))
        }
        SymbolKind::Variable => Some(format!(
            "**{}** — variable, first assigned at line {}",
            symbol.name, symbol.line
        )),
        SymbolKind::Const => Some(format!(
            "**{}** — constant, declared at line {}",
            symbol.name, symbol.line
        )),
        SymbolKind::Static => Some(format!(
            "**{}** — static variable (persists across calls), declared at line {}",
            symbol.name, symbol.line
        )),
        SymbolKind::Array { size } => Some(format!(
            "**{}** — array of size {}, declared at line {}",
            symbol.name, size, symbol.line
        )),
        SymbolKind::Import { module } => Some(format!(
            "**{}** — imported from `{}`\n\nSee module documentation for details.",
            symbol.name, module
        )),
    }
}

/// Looks up a word in the static definitions file — keywords, builtins,
/// and module functions. Returns formatted Markdown hover content or None.
fn get_static_symbol_info(word: &str) -> Option<String> {
    let word_upper = word.to_uppercase();

    if let Some(info) = DEFINITIONS.keywords.get(&word_upper) {
        return Some(format!(
            "**{}** — {}\n\n{}\n\n```psy\n{}\n```",
            word, info.description, info.detail, info.example
        ));
    }

    if let Some(info) = DEFINITIONS.builtins.get(&word_upper) {
        return Some(format!(
            "**{}** — {}\n\n{}\n\n```psy\n{}\n```",
            word, info.description, info.detail, info.example
        ));
    }

    for (module_name, module) in &DEFINITIONS.modules {
        if let Some(info) = module.functions.get(&word_upper) {
            return Some(format!(
                "**{}** — {} *(from `{}`)*\n\n{}\n\n```psy\n{}\n```",
                word, info.description, module_name, info.detail, info.example
            ));
        }
    }

    if let Some(module) = DEFINITIONS.modules.get(&word_upper) {
        let mut functions_list = String::new();
        let mut sorted_funcs: Vec<(&String, &SymbolInfo)> = module.functions.iter().collect();
        sorted_funcs.sort_by_key(|(name, _)| name.as_str());
        for (func_name, func_info) in sorted_funcs {
            functions_list.push_str(&format!("- `{}`: {}\n", func_name, func_info.description));
        }
        return Some(format!(
            "**{}** — {}\n\n{}\n\n**Available functions:**\n{}",
            word, module.description, module.detail, functions_list
        ));
    }

    None
}

fn get_word_at_position(document: &str, position: &Position) -> Option<(String, Range)> {
    let lines: Vec<&str> = document.lines().collect();
    let line = lines.get(position.line as usize)?;

    if position.character as usize > line.len() {
        return None;
    }

    let chars: Vec<char> = line.chars().collect();
    let mut start = position.character as usize;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    let mut end = position.character as usize;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    if start == end {
        return None;
    }

    let word: String = chars[start..end].iter().collect();

    Some((
        word,
        Range {
            start: Position {
                line: position.line,
                character: start as u32,
            },
            end: Position {
                line: position.line,
                character: end as u32,
            },
        },
    ))
}

fn convert_diagnostic(d: &PsyDiagnostic) -> Diagnostic {
    let line = d.line.saturating_sub(1) as u32;
    let column = d.column.saturating_sub(1) as u32;

    let severity = match d.severity {
        PsySeverity::Error => DiagnosticSeverity::ERROR,
        PsySeverity::Warning => DiagnosticSeverity::WARNING,
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

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(Mutex::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
