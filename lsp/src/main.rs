use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use psycore::parser::ast::{Expression, OutputValue, Spanned, Statement};
use syntax::symbols::Symbol;

struct StaticSymbol {
    description: &'static str,
    detail: &'static str,
    example: &'static str,
}

/// Dynamic type inferred from tracking expressions in structural context
#[derive(Debug, Clone, PartialEq)]
enum InferredType {
    Number,
    String,
    Boolean,
    Array,
    Unknown,
}

impl InferredType {
    fn to_str(&self) -> &'static str {
        match self {
            InferredType::Number => "Number",
            InferredType::String => "String",
            InferredType::Boolean => "Boolean",
            InferredType::Array => "Array",
            InferredType::Unknown => "Unknown",
        }
    }
}

/// Tracks complete lexical variable metadata, block contexts, and signatures.
struct SemanticContext {
    variable_types: HashMap<String, InferredType>,
    function_returns: HashMap<String, InferredType>,
    imported_modules: HashSet<String>,
    used_names: HashSet<String>,
}

#[derive(Default)]
struct DocumentState {
    content: String,
    symbols: Vec<Symbol>,
    variable_types: HashMap<String, InferredType>,
    function_returns: HashMap<String, InferredType>,
    imported_modules: HashSet<String>,
    used_names: HashSet<String>,
}

pub struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

// Inlined core documentation mappings replacing external JSON dependencies
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
            example: "FUNCTION calc(a, b)\n    RETURN a + b\nENDFUNCTION",
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
            example: "IMPORT _MATH [_SIN, _COS]",
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

        // 1. Check user-defined items with inferred types attached
        if let Some(symbol) = doc_state.symbols.iter().find(|s| s.name == word) {
            let hover_text = match &symbol.kind {
                syntax::symbols::SymbolKind::Function { parameters } => {
                    let ret_type = doc_state
                        .function_returns
                        .get(&word)
                        .cloned()
                        .unwrap_or(InferredType::Unknown);
                    format!(
                        "```psy\nFUNCTION {}({}) -> {}\n```\n\n*User-defined routine* — line {}",
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
                        "**{}** — Type: `{}`\n\nDeclared or tracked at line {}",
                        symbol.name,
                        var_type.to_str(),
                        symbol.line
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

        // 2. Check Static Language Keywords
        let upper_word = word.to_uppercase();
        if let Some((_, info)) = KEYWORDS.iter().find(|(k, _)| *k == upper_word) {
            let content = format!(
                "**{}** — {}\n\n{}\n\n```psy\n{}\n```",
                word, info.description, info.detail, info.example
            );
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(range),
            }));
        }

        // 3. Check Native Structural Modules and Native Functions directly
        if let Some(module) = psycore::interpreter::native::get_module(&upper_word) {
            let mut functions_list = String::new();
            for f_name in module.functions.keys() {
                functions_list.push_str(&format!("- `{}`\n", f_name));
            }
            let content = format!(
                "**{}** — Standard System Library Module\n\n**Exposed Bindings:**\n{}",
                word, functions_list
            );
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: Some(range),
            }));
        }

        // Scan inside system modules for functions matching invocation calls
        for m_name in &["_MATH", "_FS", "_TIME", "_CRYPTO"] {
            if let Some(module) = psycore::interpreter::native::get_module(m_name) {
                if module.functions.contains_key(upper_word.as_str()) {
                    let content = format!(
                        "**{}** — Native Core Standard Library Function (From `{}` module)",
                        word, m_name
                    );
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: Some(range),
                    }));
                }
                if module.constants.contains_key(upper_word.as_str()) {
                    let content = format!(
                        "**{}** — Native Built-In constant Allocation (From `{}` module)",
                        word, m_name
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

        // Standard Keyword Completions
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

        // Built-in Module Suggestions
        for m_name in &["_MATH", "_FS", "_TIME", "_CRYPTO"] {
            items.push(CompletionItem {
                label: m_name.to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(format!("Core runtime framework suite module")),
                ..Default::default()
            });

            // Automatically augment suggestions if module imports are verified active in tree state
            if doc_state.imported_modules.contains(*m_name) {
                if let Some(mod_reg) = psycore::interpreter::native::get_module(m_name) {
                    for f_name in mod_reg.functions.keys() {
                        items.push(CompletionItem {
                            label: f_name.to_string(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!("Native binding from {}", m_name)),
                            ..Default::default()
                        });
                    }
                    for c_name in mod_reg.constants.keys() {
                        items.push(CompletionItem {
                            label: c_name.to_string(),
                            kind: Some(CompletionItemKind::CONSTANT),
                            detail: Some(format!(
                                "Native static constant constant from {}",
                                m_name
                            )),
                            ..Default::default()
                        });
                    }
                }
            }
        }

        // Local Document Symbols
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
        let (ast, mut diagnostics) = syntax::parse_ast(source);
        let symbols = syntax::symbols(source);

        let mut ctx = SemanticContext {
            variable_types: HashMap::new(),
            function_returns: HashMap::new(),
            imported_modules: HashSet::new(),
            used_names: HashSet::new(),
        };

        // Recursively visit every single AST statement and nested structural expression tree
        walk_statements(&ast, &mut ctx);

        let mut lsp_diagnostics: Vec<Diagnostic> =
            diagnostics.iter().map(|d| convert_diagnostic(d)).collect();

        // Produce standard diagnostic highlights for unused imports cleanly
        for symbol in &symbols {
            if let syntax::symbols::SymbolKind::Import { module } = &symbol.kind {
                if !ctx.used_names.contains(&symbol.name) {
                    lsp_diagnostics.push(Diagnostic {
                        range: Range {
                            start: Position { line: symbol.line.saturating_sub(1) as u32, character: symbol.column.saturating_sub(1) as u32 },
                            end: Position { line: symbol.line.saturating_sub(1) as u32, character: (symbol.column.saturating_sub(1) + symbol.name.len()) as u32 },
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
                    used_names: ctx.used_names,
                },
            );
        }

        self.client
            .publish_diagnostics(uri.clone(), lsp_diagnostics, None)
            .await;
    }
}

fn walk_statements(statements: &[Spanned<Statement>], ctx: &mut SemanticContext) {
    for spanned in statements {
        match &spanned.node {
            Statement::Import { modules } => {
                for m in modules {
                    ctx.imported_modules.insert(m.name.clone());
                    if let Some(funcs) = &m.functions {
                        // Gather function references explicitly tracking imports
                        for f in funcs {
                            ctx.variable_types.insert(f.clone(), InferredType::Unknown);
                        }
                    }
                }
            }
            Statement::Assign {
                variable,
                expression,
            } => {
                let expr_type = infer_expression_type(&expression.node);
                ctx.variable_types.insert(variable.clone(), expr_type);
                walk_expression(&expression.node, ctx);
            }
            Statement::ConstDeclaration { name, expression } => {
                let expr_type = infer_expression_type(&expression.node);
                ctx.variable_types.insert(name.clone(), expr_type);
                walk_expression(&expression.node, ctx);
            }
            Statement::StaticDeclaration { name, expression } => {
                let expr_type = infer_expression_type(&expression.node);
                ctx.variable_types.insert(name.clone(), expr_type);
                walk_expression(&expression.node, ctx);
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                for param in parameters {
                    ctx.variable_types
                        .insert(param.clone(), InferredType::Unknown);
                }

                // Track deep returns inside routine container blocks to isolate function return types structural models
                let mut routine_returns = InferredType::Unknown;
                for stmt in body {
                    if let Statement::Return { value: Some(expr) } = &stmt.node {
                        routine_returns = infer_expression_type(&expr.node);
                    }
                }
                ctx.function_returns.insert(name.clone(), routine_returns);
                walk_statements(body, ctx);
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
                ctx.used_names.insert(name.clone());
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
        Expression::Identifier(name) => {
            ctx.used_names.insert(name.clone());
        }
        Expression::FunctionCall { name, arguments } => {
            ctx.used_names.insert(name.clone());
            for arg in arguments {
                walk_expression(&arg.node, ctx);
            }
        }
        Expression::BinaryOp { left, right, .. } => {
            walk_expression(&left.node, ctx);
            walk_expression(&right.node, ctx);
        }
        Expression::UnaryOp { expr, .. } => {
            walk_expression(&expr.node, ctx);
        }
        Expression::ArrayAccess { name, index } => {
            ctx.used_names.insert(name.clone());
            walk_expression(&index.node, ctx);
        }
        Expression::ArrayLiteral(elements) => {
            for el in elements {
                walk_expression(&el.node, ctx);
            }
        }
        _ => {}
    }
}

fn infer_expression_type(expr: &Expression) -> InferredType {
    match expr {
        Expression::Number(_) => InferredType::Number,
        Expression::String(_) => InferredType::String,
        Expression::Boolean(_) => InferredType::Boolean,
        Expression::ArrayLiteral(_) => InferredType::Array,
        Expression::BinaryOp { left, operator, .. } => match operator {
            psycore::parser::ast::Operator::Add
            | psycore::parser::ast::Operator::Subtract
            | psycore::parser::ast::Operator::Multiply
            | psycore::parser::ast::Operator::Divide
            | psycore::parser::ast::Operator::Modulo
            | psycore::parser::ast::Operator::Power => InferredType::Number,
            _ => InferredType::Boolean,
        },
        Expression::UnaryOp { operator, .. } => match operator {
            psycore::parser::ast::UnaryOperator::Negate => InferredType::Number,
            psycore::parser::ast::UnaryOperator::Not => InferredType::Boolean,
        },
        _ => InferredType::Unknown,
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
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
