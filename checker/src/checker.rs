use crate::diagnostics::Diagnostic;
use crate::rules;
use pseudocode_core::lexer::Lexer;
use pseudocode_core::parser::Parser;

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
