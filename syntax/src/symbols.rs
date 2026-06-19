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
            Statement::Assign { variables, .. } => {
                for variable in variables {
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
                ..
            } => {
                let param_names: Vec<String> = parameters.iter().map(|p| p.name.clone()).collect();
                push_if_new(
                    symbols,
                    Symbol {
                        name: name.clone(),
                        kind: SymbolKind::Function {
                            parameters: param_names,
                        },
                        line,
                        column,
                    },
                );
                for param in parameters {
                    push_if_new(
                        symbols,
                        Symbol {
                            name: param.name.clone(),
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
