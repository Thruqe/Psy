use crate::diagnostics::Diagnostic;
use crate::rules;
use psy_core::lexer::Lexer;
use psy_core::parser::Parser;

/// Runs the lexer + parser over `source` and returns every diagnostic
/// found, with rule-based suggestions attached where applicable.
/// Never panics on malformed input — that's the entire point of a checker.
pub fn check(source: &str) -> Vec<Diagnostic> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let (_ast, parse_errors) = parser.parse();

    parse_errors
        .into_iter()
        .map(|err| {
            let mut diagnostic: Diagnostic = err.into();
            diagnostic.suggestion = rules::suggest_fix(&diagnostic.message);
            diagnostic
        })
        .collect()
}

pub fn symbols(source: &str) -> Vec<crate::symbols::Symbol> {
    use psy_core::lexer::Lexer;
    use psy_core::parser::Parser;

    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();
    let mut parser = Parser::new(tokens);
    let (ast, _) = parser.parse();

    crate::symbols::collect_symbols(&ast)
}
