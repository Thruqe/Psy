use psy_core::parser::ParseError;
pub use psy_core::parser::Severity;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub suggestion: Option<String>,
}

impl From<ParseError> for Diagnostic {
    fn from(err: ParseError) -> Self {
        Diagnostic {
            message: err.message,
            line: err.line,
            column: err.column,
            severity: err.severity,
            suggestion: None,
        }
    }
}
