use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use psycore::parser::ast::{Expression, Operator, OutputValue, Spanned, Statement, UnaryOperator};
use syntax::symbols::Symbol;

struct StaticSymbol {
    description: &'static str,
    detail: &'static str,
    example: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
enum InferredType {
    Number,
    String,
    Boolean,
    Array,
    Void,
    Unknown,
}

impl InferredType {
    fn to_str(&self) -> &'static str {
        match self {
            InferredType::Number => "Number",
            InferredType::String => "String",
            InferredType::Boolean => "Boolean",
            InferredType::Array => "Array",
            InferredType::Void => "Void",
            InferredType::Unknown => "Unknown",
        }
    }

    fn from_str(s: &str) -> Self {
        match s {
            "Number" => InferredType::Number,
            "String" => InferredType::String,
            "Boolean" => InferredType::Boolean,
            "Array" => InferredType::Array,
            "Void" => InferredType::Void,
            _ => InferredType::Unknown,
        }
    }
}

struct SemanticContext {
    variable_types: HashMap<String, InferredType>,
    function_returns: HashMap<String, InferredType>,
    function_params: HashMap<String, Vec<(String, InferredType)>>,
    imported_modules: HashSet<String>,
    used_names: HashSet<String>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Default)]
struct DocumentState {
    content: String,
    symbols: Vec<Symbol>,
    variable_types: HashMap<String, InferredType>,
    function_returns: HashMap<String, InferredType>,
    imported_modules: HashSet<String>,
}

pub struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
    global_exports:
        Arc<Mutex<HashMap<String, (InferredType, Vec<(String, InferredType)>, String, Url)>>>,
}

const KEYWORDS: &[(&str, StaticSymbol)] = &[
    (
        "START",
        StaticSymbol {
            description: "Program marker",
            detail: "Denotes program start block initialization.",
            example: "START\n    OUTPUT \"Hello\"\nEND",
        },
    ),
    (
        "END",
        StaticSymbol {
            description: "Program termination marker",
            detail: "Denotes clean completion of the executable environment context.",
            example: "START\nEND",
        },
    ),
    (
        "IF",
        StaticSymbol {
            description: "Conditional execution branch",
            detail: "Evaluates matching conditions sequentially.",
            example: "IF test == true THEN\n    OUTPUT \"Passed\"\nENDIF",
        },
    ),
    (
        "THEN",
        StaticSymbol {
            description: "Conditional true directive",
            detail: "Binds conditional blocks closely to evaluated criteria.",
            example: "IF x == 1 THEN\n    OUTPUT x\nENDIF",
        },
    ),
    (
        "ELSE",
        StaticSymbol {
            description: "Default execution path fallback",
            detail: "Executes statements when prior criteria fail expressions.",
            example: "IF x == 1 THEN\n    OUTPUT 1\nELSE\n    OUTPUT 0\nENDIF",
        },
    ),
    (
        "ELSEIF",
        StaticSymbol {
            description: "Secondary choice block evaluation",
            detail: "Chains independent evaluations cleanly.",
            example: "IF x == 1 THEN\n    OUTPUT 1\nELSEIF x == 2 THEN\n    OUTPUT 2\nENDIF",
        },
    ),
    (
        "ENDIF",
        StaticSymbol {
            description: "Conditional terminator statement",
            detail: "Closes the block evaluation frame neatly.",
            example: "IF flag THEN\n    OUTPUT \"Yes\"\nENDIF",
        },
    ),
    (
        "FOR",
        StaticSymbol {
            description: "Definite iteration step loop",
            detail: "Iterates an tracking counter variable over an explicit bounds context.",
            example: "FOR i = 1 TO 10\n    OUTPUT i\nENDFOR",
        },
    ),
    (
        "TO",
        StaticSymbol {
            description: "Iteration endpoint criteria",
            detail: "Configures higher bounds constraints in numeric loops.",
            example: "FOR target = 0 TO 100\nENDFOR",
        },
    ),
    (
        "ENDFOR",
        StaticSymbol {
            description: "Loop boundary closer",
            detail: "Closes a definite collection iteration block loop context.",
            example: "FOR i = 1 TO 5\nENDFOR",
        },
    ),
    (
        "WHILE",
        StaticSymbol {
            description: "Indefinite iteration execution block",
            detail: "Loops matching statements iteratively while constraints compute true.",
            example: "WHILE active == true\nENDWHILE",
        },
    ),
    (
        "ENDWHILE",
        StaticSymbol {
            description: "While iteration closer",
            detail: "Closes an active running logical evaluation block sequence cleanly.",
            example: "WHILE flag\nENDWHILE",
        },
    ),
    (
        "FUNCTION",
        StaticSymbol {
            description: "Declares modular local executable routines",
            detail: "Encapsulates execution context blocks using clear argument bounds identifiers.",
            example: "FUNCTION calc(a Number, b Number) -> Number\n    RETURN a + b\nENDFUNCTION",
        },
    ),
    (
        "ENDFUNCTION",
        StaticSymbol {
            description: "Routine block closer",
            detail: "Closes active encapsulation of custom callable code environments.",
            example: "FUNCTION inline()\nENDFUNCTION",
        },
    ),
    (
        "RETURN",
        StaticSymbol {
            description: "Renders evaluated outputs out of active contexts",
            detail: "Stops execution in routine code frames immediately, throwing back results.",
            example: "RETURN static_val",
        },
    ),
    (
        "INPUT",
        StaticSymbol {
            description: "Read standard workspace terminal inputs",
            detail: "Captures user responses into defined memory storage references dynamically.",
            example: "INPUT target_var",
        },
    ),
    (
        "OUTPUT",
        StaticSymbol {
            description: "Writes evaluated values out to standard streams",
            detail: "Dumps strings, calculated values, or indices explicitly.",
            example: "OUTPUT \"Result:\", computed_val",
        },
    ),
    (
        "DECLARE",
        StaticSymbol {
            description: "Allocates uniform array collection references",
            detail: "Sets static memory boundaries for safe structural indexed access tracks.",
            example: "DECLARE matrix[10]",
        },
    ),
    (
        "CONST",
        StaticSymbol {
            description: "Immutable evaluation allocation",
            detail: "Binds identifiers tightly to singular constants that cannot change.",
            example: "CONST MAX = 100",
        },
    ),
    (
        "STATIC",
        StaticSymbol {
            description: "Preserved assignment lifetime storage",
            detail: "Retains values cleanly across subsequent invocation updates safely.",
            example: "STATIC tracking_counter = 0",
        },
    ),
    (
        "PUB",
        StaticSymbol {
            description: "Export visibility definition toggle",
            detail: "Allows other workspaces or consumer code scopes to access underlying assets.",
            example: "PUB FUNCTION shared_api()\nENDFUNCTION",
        },
    ),
    (
        "IMPORT",
        StaticSymbol {
            description: "Pulls targeted functionality systems into global environments",
            detail: "Links internal frameworks to native math, filesystem, or cryptographic modules.",
            example: "IMPORT _MATH[SIN, COS]",
        },
    ),
];

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
                document_formatting_provider: Some(OneOf::Left(true)),
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
            .log_message(MessageType::INFO, "Psy Native LSP Engine Activated.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        self.parse_and_publish(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            let uri = params.text_document.uri.clone();
            let text = change.text;
            self.parse_and_publish(&uri, &text).await;
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

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut formatter = psycore::formatter::Formatter::new();
        match formatter.format(&doc_state.content) {
            Ok(formatted_text) => {
                let lines: Vec<&str> = doc_state.content.lines().collect();
                let last_line = lines.len().saturating_sub(1) as u32;
                let last_char = lines.last().map(|l| l.len()).unwrap_or(0) as u32;

                Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: last_line,
                            character: last_char,
                        },
                    },
                    new_text: formatted_text,
                }]))
            }
            Err(_) => Ok(None),
        }
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

        // CHECK NATIVE MODULES FIRST for better hover info
        let upper_word = word.to_uppercase();
        for m_name in psycore::interpreter::native::module_names() {
            if let Some(module) = psycore::interpreter::native::get_module(m_name) {
                if let Some(func_info) = module.get_function_info(upper_word.as_str()) {
                    let params: Vec<String> = func_info
                        .parameters
                        .iter()
                        .map(|(p, t)| format!("{} {}", p, t))
                        .collect();

                    // Code-like display with syntax highlighting
                    let content = format!(
                        "```psy\n\
                     FUNCTION {}({}) -> {}\n\
                     ```\n\
                     \n\
                     *Native function from `{}` module*\n\
                     \n\
                     {}\n\
                     \n\
                     **Parameters:**\n\
                     {}\n\
                     \n\
                     **Returns:** `{}`",
                        word,
                        params.join(", "),
                        func_info.return_type,
                        module.name,
                        func_info.description,
                        if func_info.parameters.is_empty() {
                            "  • *none*".to_string()
                        } else {
                            func_info
                                .parameters
                                .iter()
                                .map(|(p, t)| format!("  • `{}`: `{}`", p, t))
                                .collect::<Vec<_>>()
                                .join("\n")
                        },
                        func_info.return_type
                    );
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(range),
                    }));
                }
                if let Some(const_info) = module.get_constant_info(upper_word.as_str()) {
                    let content = format!(
                        "```psy\n\
                     CONST {}: {}\n\
                     ```\n\
                     \n\
                     *Native constant from `{}` module*\n\
                     \n\
                     {}\n\
                     \n\
                     **Type:** `{}`",
                        word,
                        const_info.constant_type,
                        module.name,
                        const_info.description,
                        const_info.constant_type
                    );
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(range),
                    }));
                }
            }
        }

        // THEN check document symbols (for user-defined variables/functions)
        if let Some(symbol) = doc_state.symbols.iter().find(|s| s.name == word) {
            let hover_text = match &symbol.kind {
                syntax::symbols::SymbolKind::Function { parameters } => {
                    let ret_type = doc_state
                        .function_returns
                        .get(&word)
                        .cloned()
                        .unwrap_or(InferredType::Unknown);
                    format!(
                        "```psy\n\
                     FUNCTION {}({}) -> {}\n\
                     ```\n\
                     \n\
                     *User-defined routine*\n\
                     \n\
                     **Declared at:** line {}",
                        symbol.name,
                        parameters.join(", "),
                        ret_type.to_str(),
                        symbol.line
                    )
                }
                _ => {
                    let var_type = doc_state
                        .variable_types
                        .get(&word)
                        .cloned()
                        .unwrap_or(InferredType::Unknown);
                    format!(
                        "```psy\n\
                     {}: {}\n\
                     ```\n\
                     \n\
                     *Variable declaration*\n\
                     \n\
                     **Declared at:** line {}\n\
                     **Type:** `{}`",
                        symbol.name,
                        var_type.to_str(),
                        symbol.line,
                        var_type.to_str()
                    )
                }
            };
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: Some(range),
            }));
        }

        // Then check global exports
        {
            let exports = self.global_exports.lock().await;
            if let Some((ret_type, params, kind_str, _)) = exports.get(&word) {
                let hover_text = if kind_str == "FUNCTION" {
                    let param_strs: Vec<String> = params
                        .iter()
                        .map(|(p, t)| format!("{} {}", p, t.to_str()))
                        .collect();
                    format!(
                        "```psy\n\
                     PUB FUNCTION {}({}) -> {}\n\
                     ```\n\
                     \n\
                     *Shared Foreign Export*\n\
                     \n\
                     **Parameters:**\n\
                     {}\n\
                     \n\
                     **Returns:** `{}`",
                        word,
                        param_strs.join(", "),
                        ret_type.to_str(),
                        if params.is_empty() {
                            "  • *none*".to_string()
                        } else {
                            params
                                .iter()
                                .map(|(p, t)| format!("  • `{}`: `{}`", p, t.to_str()))
                                .collect::<Vec<_>>()
                                .join("\n")
                        },
                        ret_type.to_str()
                    )
                } else {
                    format!(
                        "```psy\n\
                     PUB {} {}: {}\n\
                     ```\n\
                     \n\
                     *Shared Foreign Export*\n\
                     \n\
                     **Type:** `{}`",
                        kind_str,
                        word,
                        ret_type.to_str(),
                        ret_type.to_str()
                    )
                };
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: Some(range),
                }));
            }
        }

        // Check keywords
        if let Some((_, info)) = KEYWORDS.iter().find(|(k, _)| *k == upper_word) {
            let content = format!(
                "```psy\n{}\n```\n\n**{}** — Keyword\n\n{}\n\n{}",
                info.example, word, info.description, info.detail
            );
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(range),
            }));
        }

        // Check if hovering over a module name
        if let Some(module) = psycore::interpreter::native::get_module(&upper_word) {
            let mut functions_list = String::new();
            for (f_name, f_info) in &module.functions {
                let params: Vec<String> = f_info
                    .parameters
                    .iter()
                    .map(|(p, t)| format!("{} {}", p, t))
                    .collect();
                functions_list.push_str(&format!(
                    "  • `{}({}) -> {}`\n",
                    f_name,
                    params.join(", "),
                    f_info.return_type
                ));
            }
            for c_name in module.constants.keys() {
                functions_list.push_str(&format!("  • `{}` (constant)\n", c_name));
            }
            let content = format!(
                "**{}** — Native Module\n\n{}\n\n**Exposed Bindings:**\n{}",
                word, module.description, functions_list
            );
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(range),
            }));
        }

        Ok(None)
    }
    
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let docs = self.documents.lock().await;
        let doc_state = match docs.get(&uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let mut items = Vec::new();

        for (kw, info) in KEYWORDS {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(info.description.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("{}\n\n```psy\n{}\n```", info.detail, info.example),
                })),
                ..Default::default()
            });
        }

        let current_line = params.text_document_position.position.line;
        for m_name in psycore::interpreter::native::module_names() {
            if !doc_state.imported_modules.contains(m_name) {
                let module_desc = psycore::interpreter::native::get_module(m_name)
                    .map(|m| m.description.to_string())
                    .unwrap_or_default();
                items.push(CompletionItem {
                    label: format!("IMPORT {}", m_name),
                    kind: Some(CompletionItemKind::MODULE),
                    detail: Some(module_desc),
                    additional_text_edits: Some(vec![TextEdit {
                        range: Range {
                            start: Position {
                                line: 0,
                                character: 0,
                            },
                            end: Position {
                                line: 0,
                                character: 0,
                            },
                        },
                        new_text: format!("IMPORT {}[]\n", m_name),
                    }]),
                    text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                        range: Range {
                            start: Position {
                                line: current_line,
                                character: params
                                    .text_document_position
                                    .position
                                    .character
                                    .saturating_sub(1),
                            },
                            end: Position {
                                line: current_line,
                                character: params.text_document_position.position.character,
                            },
                        },
                        new_text: "".to_string(),
                    })),
                    ..Default::default()
                });
            }

            if doc_state.imported_modules.contains(m_name) {
                if let Some(mod_reg) = psycore::interpreter::native::get_module(m_name) {
                    for (f_name, f_info) in &mod_reg.functions {
                        if doc_state.variable_types.contains_key(&f_name[..]) {
                            continue;
                        }
                        let params: Vec<String> = f_info
                            .parameters
                            .iter()
                            .map(|(p, t)| format!("{}: {}", p, t))
                            .collect();
                        items.push(CompletionItem {
                            label: f_name.to_string(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!(
                                "{} -> {}",
                                params.join(", "),
                                f_info.return_type
                            )),
                            documentation: Some(Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!(
                                    "**{}**\n\n{}\n\n**Returns:** {}",
                                    f_name, f_info.description, f_info.return_type
                                ),
                            })),
                            ..Default::default()
                        });
                    }
                    for (c_name, c_info) in &mod_reg.constants {
                        if doc_state.variable_types.contains_key(&c_name[..]) {
                            continue;
                        }
                        items.push(CompletionItem {
                            label: c_name.to_string(),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some(format!("{}: {}", c_name, c_info.constant_type)),
                            documentation: Some(Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: format!("**{}**\n\n{}", c_name, c_info.description),
                            })),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        {
            let exports = self.global_exports.lock().await;
            for (exp_name, (ret_type, _, kind_str, _)) in exports.iter() {
                if !doc_state.variable_types.contains_key(exp_name) {
                    let item_kind = match kind_str.as_str() {
                        "FUNCTION" => CompletionItemKind::FUNCTION,
                        "CONST" => CompletionItemKind::CONSTANT,
                        "STATIC" => CompletionItemKind::VARIABLE,
                        _ => CompletionItemKind::REFERENCE,
                    };
                    items.push(CompletionItem {
                        label: exp_name.clone(),
                        kind: Some(item_kind),
                        detail: Some(format!(
                            "Global PUB {} (Type: {})",
                            kind_str,
                            ret_type.to_str()
                        )),
                        ..Default::default()
                    });
                }
            }
        }

        for symbol in &doc_state.symbols {
            let kind = match &symbol.kind {
                syntax::symbols::SymbolKind::Function { .. } => CompletionItemKind::FUNCTION,
                syntax::symbols::SymbolKind::Const => CompletionItemKind::CONSTANT,
                _ => CompletionItemKind::VARIABLE,
            };
            items.push(CompletionItem {
                label: symbol.name.clone(),
                kind: Some(kind),
                ..Default::default()
            });
        }

        let mut seen = HashSet::new();
        items.retain(|item| seen.insert(item.label.clone()));

        Ok(Some(CompletionResponse::Array(items)))
    }
}

impl Backend {
    async fn parse_and_publish(&self, uri: &Url, source: &str) {
        let (ast, diagnostics) = syntax::parse_ast(source);
        let symbols = syntax::symbols(source);

        let mut ctx = SemanticContext {
            variable_types: HashMap::new(),
            function_returns: HashMap::new(),
            function_params: HashMap::new(),
            imported_modules: HashSet::new(),
            used_names: HashSet::new(),
            diagnostics: Vec::new(),
        };

        {
            let mut exports = self.global_exports.lock().await;
            exports.retain(|_, (_, _, _, origin_url)| origin_url != uri);
        }

        {
            let exports = self.global_exports.lock().await;
            for (exp_name, (ret_type, params, _, _)) in exports.iter() {
                ctx.variable_types
                    .insert(exp_name.clone(), ret_type.clone());
                ctx.function_returns
                    .insert(exp_name.clone(), ret_type.clone());
                ctx.function_params.insert(exp_name.clone(), params.clone());
            }
        }

        let mut block_execution_started = false;
        for spanned_stmt in &ast {
            match &spanned_stmt.node {
                Statement::Import { .. } => {
                    if block_execution_started {
                        ctx.diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: spanned_stmt.line.saturating_sub(1) as u32, character: spanned_stmt.column.saturating_sub(1) as u32 },
                                end: Position { line: spanned_stmt.line.saturating_sub(1) as u32, character: (spanned_stmt.column.saturating_sub(1) + 6) as u32 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("psy-analysis".to_string()),
                            message: "Layout Constraint Error: IMPORT statements must appear at the top of the file before any block initializations.".to_string(),
                            ..Default::default()
                        });
                    }
                }
                _ => {
                    block_execution_started = true;
                }
            }
        }

        walk_statements(&ast, &mut ctx);

        {
            let mut exports = self.global_exports.lock().await;
            for spanned_stmt in &ast {
                if let Statement::Public(inner_stmt) = &spanned_stmt.node {
                    match &inner_stmt.node {
                        Statement::FunctionDeclaration {
                            name,
                            parameters,
                            return_type,
                            body: _,
                        } => {
                            let ret = match return_type {
                                Some(t) => InferredType::from_str(t),
                                None => InferredType::Void,
                            };
                            let params_mappings = parameters
                                .iter()
                                .map(|p| {
                                    let t = p
                                        .data_type
                                        .as_ref()
                                        .map(|s| InferredType::from_str(s))
                                        .unwrap_or(InferredType::Unknown);
                                    (p.name.clone(), t)
                                })
                                .collect();
                            exports.insert(
                                name.clone(),
                                (ret, params_mappings, "FUNCTION".to_string(), uri.clone()),
                            );
                        }
                        Statement::ConstDeclaration { name, expression } => {
                            let t = infer_expression_type(&expression.node, &ctx);
                            exports.insert(
                                name.clone(),
                                (t, Vec::new(), "CONST".to_string(), uri.clone()),
                            );
                        }
                        Statement::StaticDeclaration { name, expression } => {
                            let t = infer_expression_type(&expression.node, &ctx);
                            exports.insert(
                                name.clone(),
                                (t, Vec::new(), "STATIC".to_string(), uri.clone()),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        let mut lsp_diagnostics: Vec<Diagnostic> =
            diagnostics.iter().map(|d| convert_diagnostic(d)).collect();

        lsp_diagnostics.extend(ctx.diagnostics);

        let source_lines: Vec<&str> = source.lines().collect();

        for symbol in &symbols {
            if let syntax::symbols::SymbolKind::Import { module } = &symbol.kind {
                let symbol_upper = symbol.name.to_uppercase();
                if !ctx.used_names.contains(&symbol.name) && !ctx.used_names.contains(&symbol_upper)
                {
                    let start_line = symbol.line.saturating_sub(1) as u32;
                    let mut start_col = symbol.column.saturating_sub(1) as u32;

                    if let Some(line_text) = source_lines.get(start_line as usize) {
                        if let Some(byte_offset) = line_text.find(&symbol.name) {
                            start_col = byte_offset as u32;
                        }
                    }

                    lsp_diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: start_line, character: start_col },
                            end: Position { line: start_line, character: start_col + symbol.name.len() as u32 },
                        },
                        severity: Some(DiagnosticSeverity::WARNING),
                        source: Some("psy-analysis".to_string()),
                        message: format!("Imported routine binding '{}' from module {} is defined but never used.", symbol.name, module),
                        tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                        ..Default::default()
                    });
                }
            }
        }

        {
            let mut docs = self.documents.lock().await;
            docs.insert(
                uri.clone(),
                DocumentState {
                    content: source.to_string(),
                    symbols,
                    variable_types: ctx.variable_types,
                    function_returns: ctx.function_returns,
                    imported_modules: ctx.imported_modules,
                },
            );
        }

        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }
}

fn collect_return_types(
    statements: &[Spanned<Statement>],
    ctx: &SemanticContext,
    types: &mut Vec<InferredType>,
) {
    for spanned in statements {
        match &spanned.node {
            Statement::Return { value: Some(expr) } => {
                types.push(infer_expression_type(&expr.node, ctx));
            }
            Statement::Return { value: None } => {
                types.push(InferredType::Void);
            }
            Statement::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                collect_return_types(then_branch, ctx, types);
                for (_, branch) in else_if_branches {
                    collect_return_types(branch, ctx, types);
                }
                collect_return_types(else_branch, ctx, types);
            }
            Statement::ForLoop { body, .. } => {
                collect_return_types(body, ctx, types);
            }
            Statement::WhileLoop { body, .. } => {
                collect_return_types(body, ctx, types);
            }
            Statement::Public(inner) => {
                collect_return_types(&[*inner.clone()], ctx, types);
            }
            _ => {}
        }
    }
}

fn walk_statements(statements: &[Spanned<Statement>], ctx: &mut SemanticContext) {
    for spanned in statements {
        match &spanned.node {
            Statement::Import { modules } => {
                for m in modules {
                    ctx.imported_modules.insert(m.name.clone());
                    let upper_mod = m.name.to_uppercase();
                    let native_module = psycore::interpreter::native::get_module(&upper_mod);

                    if let Some(funcs) = &m.functions {
                        for f in funcs {
                            // Try to get the actual type from the native module
                            let inferred_type = if let Some(ref mod_reg) = native_module {
                                if let Some(func_info) = mod_reg.get_function_info(f.as_str()) {
                                    InferredType::from_str(func_info.return_type)
                                } else if let Some(const_info) =
                                    mod_reg.get_constant_info(f.as_str())
                                {
                                    InferredType::from_str(const_info.constant_type)
                                } else {
                                    InferredType::Unknown
                                }
                            } else {
                                InferredType::Unknown
                            };

                            ctx.variable_types.insert(f.clone(), inferred_type.clone());
                            ctx.variable_types.insert(f.to_uppercase(), inferred_type);
                            // Also add to function_returns for native functions
                            if let Some(ref mod_reg) = native_module {
                                if let Some(func_info) = mod_reg.get_function_info(f.as_str()) {
                                    ctx.function_returns.insert(
                                        f.clone(),
                                        InferredType::from_str(func_info.return_type),
                                    );
                                    ctx.function_returns.insert(
                                        f.to_uppercase(),
                                        InferredType::from_str(func_info.return_type),
                                    );
                                }
                            }

                            // Check if the function actually exists in the module
                            if let Some(ref mod_reg) = native_module {
                                if !mod_reg.has_function(f.as_str())
                                    && !mod_reg.has_function(f.to_uppercase().as_str())
                                {
                                    ctx.diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: spanned.line.saturating_sub(1) as u32, character: spanned.column.saturating_sub(1) as u32 },
                                end: Position { line: spanned.line.saturating_sub(1) as u32, character: (spanned.column.saturating_sub(1) + 6) as u32 },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("psy-analysis".to_string()),
                            message: format!("Signature Error: Binding '{}' does not exist inside core system module '{}'.", f, m.name),
                            ..Default::default()
                        });
                                }
                            }
                        }
                    }
                }
            }
            Statement::Assign {
                variables,
                expression,
            } => {
                let expr_type = infer_expression_type(&expression.node, ctx);
                for var in variables {
                    ctx.variable_types.insert(var.clone(), expr_type.clone());
                }
                walk_expression(&expression.node, ctx);
            }
            Statement::ConstDeclaration { name, expression } => {
                let expr_type = infer_expression_type(&expression.node, ctx);
                ctx.variable_types.insert(name.clone(), expr_type);
                walk_expression(&expression.node, ctx);
            }
            Statement::StaticDeclaration { name, expression } => {
                let expr_type = infer_expression_type(&expression.node, ctx);
                ctx.variable_types.insert(name.clone(), expr_type);
                walk_expression(&expression.node, ctx);
            }
            Statement::DeclareArray { name, .. } => {
                ctx.variable_types.insert(name.clone(), InferredType::Array);
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
            } => {
                let mut param_mappings = Vec::new();
                for param in parameters {
                    let p_type = match &param.data_type {
                        Some(t) => InferredType::from_str(t),
                        None => InferredType::Unknown,
                    };
                    ctx.variable_types
                        .insert(param.name.clone(), p_type.clone());
                    param_mappings.push((param.name.clone(), p_type));
                }
                ctx.function_params.insert(name.clone(), param_mappings);

                let final_return = match return_type {
                    Some(t) => InferredType::from_str(t),
                    None => {
                        let mut collected_types = Vec::new();
                        collect_return_types(body, ctx, &mut collected_types);
                        collected_types.retain(|t| *t != InferredType::Unknown);
                        if collected_types.is_empty() {
                            InferredType::Void
                        } else {
                            let first = collected_types[0].clone();
                            if collected_types.iter().all(|t| *t == first) {
                                first
                            } else {
                                InferredType::Unknown
                            }
                        }
                    }
                };

                ctx.function_returns
                    .insert(name.clone(), final_return.clone());
                walk_statements(body, ctx);

                if let Some(t_str) = return_type {
                    let expected = InferredType::from_str(t_str);
                    let mut collected_types = Vec::new();
                    collect_return_types(body, ctx, &mut collected_types);
                    for found in collected_types {
                        if found != InferredType::Unknown && found != expected {
                            ctx.diagnostics.push(Diagnostic {
                                range: Range {
                                    start: Position { line: spanned.line.saturating_sub(1) as u32, character: spanned.column.saturating_sub(1) as u32 },
                                    end: Position { line: spanned.line.saturating_sub(1) as u32, character: (spanned.column.saturating_sub(1) + name.len()) as u32 },
                                },
                                severity: Some(DiagnosticSeverity::ERROR),
                                source: Some("psy-typecheck".to_string()),
                                message: format!("Function '{}' declares return constraint '{}' but breaks contract by returning type '{}'.", name, t_str, found.to_str()),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
            Statement::If {
                condition,
                then_branch,
                else_if_branches,
                else_branch,
            } => {
                walk_expression(&condition.node, ctx);
                walk_statements(then_branch, ctx);
                for (c, b) in else_if_branches {
                    walk_expression(&c.node, ctx);
                    walk_statements(b, ctx);
                }
                walk_statements(else_branch, ctx);
            }
            Statement::ForLoop {
                variable,
                start,
                end,
                body,
            } => {
                ctx.variable_types
                    .insert(variable.clone(), InferredType::Number);
                walk_expression(&start.node, ctx);
                walk_expression(&end.node, ctx);
                walk_statements(body, ctx);
            }
            Statement::WhileLoop { condition, body } => {
                walk_expression(&condition.node, ctx);
                walk_statements(body, ctx);
            }
            Statement::Return { value: Some(expr) } => {
                walk_expression(&expr.node, ctx);
            }
            Statement::ExpressionStatement(expr) => {
                walk_expression(&expr.node, ctx);
            }
            Statement::Output { values } => {
                for val in values {
                    if let OutputValue::Expression(e) = val {
                        walk_expression(&e.node, ctx);
                    }
                }
            }
            Statement::ArrayAssign { name, index, value } => {
                ctx.variable_types.insert(name.clone(), InferredType::Array);
                walk_expression(&index.node, ctx);
                walk_expression(&value.node, ctx);
            }
            Statement::Public(inner) => {
                walk_statements(&[*inner.clone()], ctx);
            }
            _ => {}
        }
    }
}

fn walk_expression(expr: &Expression, ctx: &mut SemanticContext) {
    match expr {
        Expression::Number(_)
        | Expression::String(_)
        | Expression::Boolean(_)
        | Expression::ArrayLiteral(_) => {}
        Expression::Identifier(name) => {
            ctx.used_names.insert(name.clone());
            ctx.used_names.insert(name.to_uppercase());
        }
        Expression::FunctionCall { name, arguments } => {
            ctx.used_names.insert(name.clone());
            ctx.used_names.insert(name.to_uppercase());
            for arg in arguments {
                walk_expression(&arg.node, ctx);
            }

            let upper = name.to_uppercase();

            // Check if the function is from an imported native module
            let mut matched_native_module = false;
            for m_name in &ctx.imported_modules {
                let upper_mod = m_name.to_uppercase();
                if let Some(module) = psycore::interpreter::native::get_module(&upper_mod) {
                    if module.has_function(&upper) || module.has_constant(&upper) {
                        matched_native_module = true;
                        break;
                    }
                }
            }

            if ctx.function_params.contains_key(name) {
                if let Some(expected_params) = ctx.function_params.get(name).cloned() {
                    if arguments.len() != expected_params.len() {
                        ctx.diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: 0,
                                    character: 0,
                                },
                            },
                            severity: Some(DiagnosticSeverity::ERROR),
                            source: Some("psy-typecheck".to_string()),
                            message: format!(
                                "Routines call '{}' requires exactly {} inputs, but received {}.",
                                name,
                                expected_params.len(),
                                arguments.len()
                            ),
                            ..Default::default()
                        });
                    } else {
                        for (i, arg) in arguments.iter().enumerate() {
                            let arg_type = infer_expression_type(&arg.node, ctx);
                            let (_, param_type) = &expected_params[i];
                            if *param_type != InferredType::Unknown
                                && arg_type != InferredType::Unknown
                                && arg_type != *param_type
                            {
                                ctx.diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: arg.line.saturating_sub(1) as u32, character: arg.column.saturating_sub(1) as u32 },
                                        end: Position { line: arg.line.saturating_sub(1) as u32, character: (arg.column.saturating_sub(1) + 1) as u32 },
                                    },
                                    severity: Some(DiagnosticSeverity::ERROR),
                                    source: Some("psy-typecheck".to_string()),
                                    message: format!("Argument position {} expected type '{}' but instead evaluated to '{}'.", i + 1, param_type.to_str(), arg_type.to_str()),
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            } else if ctx.variable_types.contains_key(name)
                || ctx.variable_types.contains_key(&upper)
                || matched_native_module
            {
                // Resolved successfully
            } else {
                ctx.diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("psy-analysis".to_string()),
                    message: format!("Unresolved Symbol Error: Call reference '{}' does not match any local or foreign PUB declarations.", name),
                    ..Default::default()
                });
            }
        }
        Expression::BinaryOp {
            left,
            operator,
            right,
        } => {
            walk_expression(&left.node, ctx);
            walk_expression(&right.node, ctx);

            let left_type = infer_expression_type(&left.node, ctx);
            let right_type = infer_expression_type(&right.node, ctx);

            if left_type != InferredType::Unknown
                && right_type != InferredType::Unknown
                && left_type != right_type
            {
                let op_str = match operator {
                    Operator::Add => "+",
                    Operator::Subtract => "-",
                    Operator::Multiply => "*",
                    Operator::Divide => "/",
                    Operator::Modulo => "%",
                    Operator::Power => "^",
                    Operator::Equal => "==",
                    Operator::NotEqual => "!=",
                    Operator::LessThan => "<",
                    Operator::GreaterThan => ">",
                    Operator::LessEqual => "<=",
                    Operator::GreaterEqual => ">=",
                    Operator::And => "AND",
                    Operator::Or => "OR",
                };

                ctx.diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: left.line.saturating_sub(1) as u32, character: left.column.saturating_sub(1) as u32 },
                        end: Position { line: right.line.saturating_sub(1) as u32, character: (right.column.saturating_sub(1) + 1) as u32 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("psy-typecheck".to_string()),
                    message: format!("Type mismatch: Cannot compare or operate '{}' and '{}' using the '{}' operator.", left_type.to_str(), right_type.to_str(), op_str),
                    ..Default::default()
                });
            }
        }
        Expression::UnaryOp {
            operator: _,
            expr: _expr,
        } => {
            walk_expression(&_expr.node, ctx);
        }
        Expression::ArrayAccess { name, index } => {
            ctx.used_names.insert(name.clone());
            ctx.used_names.insert(name.to_uppercase());
            walk_expression(&index.node, ctx);

            let index_type = infer_expression_type(&index.node, ctx);
            if index_type != InferredType::Unknown && index_type != InferredType::Number {
                ctx.diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: index.line.saturating_sub(1) as u32,
                            character: index.column.saturating_sub(1) as u32,
                        },
                        end: Position {
                            line: index.line.saturating_sub(1) as u32,
                            character: (index.column.saturating_sub(1) + 1) as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("psy-typecheck".to_string()),
                    message: format!(
                        "Array index must evaluate to a Number, but found '{}'.",
                        index_type.to_str()
                    ),
                    ..Default::default()
                });
            }
        }
    }
}

fn infer_expression_type(expr: &Expression, ctx: &SemanticContext) -> InferredType {
    match expr {
        Expression::Number(_) => InferredType::Number,
        Expression::String(_) => InferredType::String,
        Expression::Boolean(_) => InferredType::Boolean,
        Expression::ArrayLiteral(_) => InferredType::Array,
        Expression::Identifier(name) => ctx
            .variable_types
            .get(name)
            .cloned()
            .unwrap_or(InferredType::Unknown),
        Expression::ArrayAccess { .. } => InferredType::Number,
        Expression::FunctionCall { name, .. } => {
            let upper = name.to_uppercase();

            // Check native modules for return type information
            for m_name in psycore::interpreter::native::module_names() {
                if let Some(module) = psycore::interpreter::native::get_module(m_name) {
                    if let Some(func_info) = module.get_function_info(&upper) {
                        return InferredType::from_str(func_info.return_type);
                    }
                }
            }

            // Fall back to local function returns
            ctx.function_returns
                .get(name)
                .cloned()
                .unwrap_or(InferredType::Unknown)
        }
        Expression::BinaryOp {
            left,
            operator,
            right,
        } => match operator {
            Operator::Add => {
                let left_t = infer_expression_type(&left.node, ctx);
                let right_t = infer_expression_type(&right.node, ctx);
                if left_t == InferredType::String || right_t == InferredType::String {
                    InferredType::String
                } else if left_t == InferredType::Number && right_t == InferredType::Number {
                    InferredType::Number
                } else {
                    InferredType::Unknown
                }
            }
            Operator::Subtract
            | Operator::Multiply
            | Operator::Divide
            | Operator::Modulo
            | Operator::Power => InferredType::Number,
            _ => InferredType::Boolean,
        },
        Expression::UnaryOp {
            operator,
            expr: _expr,
        } => match operator {
            UnaryOperator::Negate => InferredType::Number,
            UnaryOperator::Not => InferredType::Boolean,
        },
    }
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

fn convert_diagnostic(d: &syntax::diagnostics::Diagnostic) -> Diagnostic {
    let line = d.line.saturating_sub(1) as u32;
    let column = d.column.saturating_sub(1) as u32;
    let severity = match d.severity {
        psycore::parser::Severity::Error => DiagnosticSeverity::ERROR,
        psycore::parser::Severity::Warning => DiagnosticSeverity::WARNING,
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
        source: Some("psy-syntax".to_string()),
        message,
        ..Default::default()
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(Mutex::new(HashMap::new())),
        global_exports: Arc::new(Mutex::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
