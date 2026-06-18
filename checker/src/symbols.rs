use pseudocode_core::parser::ast::Statement;

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

/// Walks the parsed statements (top-level program, or a function body)
/// and collects every symbol declared at that level and within nested
/// blocks (IF/FOR/WHILE), tracking function scope for STATIC variables
/// along the way.
///
/// This intentionally does NOT track line/column for every symbol kind
/// perfectly — DeclareArray/ConstDeclaration/StaticDeclaration carry no
/// position info in the current AST (Statement nodes aren't tagged with
/// source spans), so for now position defaults to (0, 0) for those.
/// Only diagnostics carry real positions today; full go-to-definition
/// support later would require tagging AST nodes with spans during
/// parsing, which is a separate, larger change.
pub fn collect_symbols(statements: &[Statement]) -> Vec<Symbol> {
    let mut symbols = Vec::new();
    walk(statements, &mut symbols, None);
    symbols
}

fn walk(statements: &[Statement], symbols: &mut Vec<Symbol>, current_function: Option<&str>) {
    for stmt in statements {
        match unwrap_public(stmt) {
            Statement::Assign { variable, .. } => {
                symbols.push(Symbol {
                    name: variable.clone(),
                    kind: SymbolKind::Variable,
                    line: 0,
                    column: 0,
                });
            }
            Statement::Input { variables } => {
                for v in variables {
                    symbols.push(Symbol {
                        name: v.clone(),
                        kind: SymbolKind::Variable,
                        line: 0,
                        column: 0,
                    });
                }
            }
            Statement::DeclareArray { name, size } => {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Array { size: *size },
                    line: 0,
                    column: 0,
                });
            }
            Statement::ConstDeclaration { name, .. } => {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Const,
                    line: 0,
                    column: 0,
                });
            }
            Statement::StaticDeclaration { name, .. } => {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Static,
                    line: 0,
                    column: 0,
                });
            }
            Statement::FunctionDeclaration {
                name,
                parameters,
                body,
            } => {
                symbols.push(Symbol {
                    name: name.clone(),
                    kind: SymbolKind::Function {
                        parameters: parameters.clone(),
                    },
                    line: 0,
                    column: 0,
                });
                // Parameters themselves are also valid identifiers
                // within the function's own body.
                for param in parameters {
                    symbols.push(Symbol {
                        name: param.clone(),
                        kind: SymbolKind::Variable,
                        line: 0,
                        column: 0,
                    });
                }
                walk(body, symbols, Some(name));
            }
            Statement::Import { modules } => {
                for module_import in modules {
                    if let Some(names) = &module_import.functions {
                        for name in names {
                            symbols.push(Symbol {
                                name: name.clone(),
                                kind: SymbolKind::Import {
                                    module: module_import.name.clone(),
                                },
                                line: 0,
                                column: 0,
                            });
                        }
                    }
                    // A bare `IMPORT _MATH` with no bracket list imports
                    // everything from that module, but we don't have a
                    // static registry listing here in the checker crate
                    // (that lives in core::interpreter::native, which
                    // the checker doesn't depend on). For now, bare
                    // imports aren't expanded into individual symbols —
                    // worth revisiting if that gap matters in practice.
                }
            }
            Statement::ForLoop { variable, body, .. } => {
                symbols.push(Symbol {
                    name: variable.clone(),
                    kind: SymbolKind::Variable,
                    line: 0,
                    column: 0,
                });
                walk(body, symbols, current_function);
            }
            Statement::WhileLoop { body, .. } => {
                walk(body, symbols, current_function);
            }
            Statement::If {
                then_branch,
                else_if_branches,
                else_branch,
                ..
            } => {
                walk(then_branch, symbols, current_function);
                for (_, branch) in else_if_branches {
                    walk(branch, symbols, current_function);
                }
                walk(else_branch, symbols, current_function);
            }
            _ => {}
        }
    }
}

fn unwrap_public(stmt: &Statement) -> &Statement {
    match stmt {
        Statement::Public(inner) => inner,
        other => other,
    }
}