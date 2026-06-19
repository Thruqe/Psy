use psycore::parser::ast::{Spanned, Statement};

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Variable,
    Array { size: usize },
    Const,
    Static,
    Function { parameters: Vec<String> },
    Import { module: String },
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub column: usize,
}

/// Walks the parsed statements and collects every symbol declared,
/// including inside nested IF/FOR/WHILE blocks and function bodies,
/// with real source positions from the Spanned AST nodes.
pub fn collect_symbols(statements: &[Spanned<Statement>]) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    walk(statements, &mut symbols);
    symbols
}

fn walk(statements: &[Spanned<Statement>], symbols: &mut Vec<Symbol>) {
    for spanned in statements {
        let line = spanned.line;
        let column = spanned.column;

        match unwrap_public(&spanned.node) {
            Statement::Assign { variable, .. } => {
                push_if_new(
                    symbols,
                    Symbol {
                        name: variable.clone(),
                        kind: SymbolKind::Variable,
                        line,
                        column,
                    },
                );
            }
            Statement::Input { variables } => {
                for v in variables {
                    push_if_new(
                        symbols,
                        Symbol {
                            name: v.clone(),
                            kind: SymbolKind::Variable,
                            line,
                            column,
                        },
                    );
                }
            }
            Statement::DeclareArray { name, size } => {
                push_if_new(
                    symbols,
                    Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Array { size: *size },
                        line,
                        column,
                    },
                );
            }
            Statement::ConstDeclaration { name, .. } => {
                push_if_new(
                    symbols,
                    Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Const,
                        line,
                        column,
                    },
                );
            }
            Statement::StaticDeclaration { name, .. } => {
                push_if_new(
                    symbols,
                    Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Static,
                        line,
                        column,
                    },
                );
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                push_if_new(
                    symbols,
                    Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Function {
                            parameters: parameters.clone(),
                        },
                        line,
                        column,
                    },
                );
                // Parameters are valid identifiers within the function body
                for param in parameters {
                    push_if_new(
                        symbols,
                        Symbol {
                            name: param.clone(),
                            kind: SymbolKind::Variable,
                            line,
                            column,
                        },
                    );
                }
                walk(body, symbols);
            }
            Statement::Import { modules } => {
                for module_import in modules {
                    if let Some(names) = &module_import.functions {
                        for func_name in names {
                            push_if_new(
                                symbols,
                                Symbol {
                                    name: func_name.clone(),
                                    kind: SymbolKind::Import {
                                        module: module_import.name.clone(),
                                    },
                                    line,
                                    column,
                                },
                            );
                        }
                    }
                    // bare IMPORT _MATH (no bracket list) is handled
                    // separately since we'd need the native registry
                    // to enumerate all exported names — not available
                    // in the syntax crate today.
                }
            }
            Statement::ForLoop { variable, body, .. } => {
                push_if_new(
                    symbols,
                    Symbol {
                        name: variable.clone(),
                        kind: SymbolKind::Variable,
                        line,
                        column,
                    },
                );
                walk(body, symbols);
            }
            Statement::WhileLoop { body, .. } => {
                walk(body, symbols);
            }
            Statement::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                walk(then_branch, symbols);
                for (_, branch) in else_if_branches {
                    walk(branch, symbols);
                }
                walk(else_branch, symbols);
            }
            _ => {}
        }
    }
}

/// Only pushes a symbol if no symbol with the same name already exists,
/// since the same variable name may be assigned multiple times (e.g. in
/// a loop) and we only want the first declaration site.
fn push_if_new(symbols: &mut Vec<Symbol>, symbol: Symbol) {
    if !symbols.iter().any(|s| s.name == symbol.name) {
        symbols.push(symbol);
    }
}

fn unwrap_public(stmt: &Statement) -> &Statement {
    match stmt {
        Statement::Public(inner) => &inner.node,
        other => other,
    }
}
