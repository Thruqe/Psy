use psycore::parser::Severity;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub message: String,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub suggestion: Option<String>,
}

impl From<psycore::parser::ParseError> for Diagnostic {
    fn from(error: psycore::parser::ParseError) -> Self {
        let msg = error.message.clone();
        let mut suggestion = None;

        if msg.contains("Expected = for assignment") {
            suggestion = Some("Assignments use a single = (not == or :=)".to_string());
        } else if msg.contains("Type mismatch") {
            suggestion = Some("Ensure both operands match the same primitive data type (e.g., both Numbers or both Strings)".to_string());
        } else if msg.contains("Array index must evaluate to a Number") {
            suggestion = Some("Use an integer literal, numerical variable, or an expression evaluating to a Number as the index bracket boundary".to_string());
        } else if msg.contains("declares return constraint") {
            suggestion = Some("Modify the expression following your RETURN keyword to match the declared explicit function signature type".to_string());
        }

        Diagnostic {
            message: msg,
            line: error.line,
            column: error.column,
            severity: error.severity,
            suggestion,
        }
    }
}
