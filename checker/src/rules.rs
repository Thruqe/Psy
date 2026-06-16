/// Maps known parse-error message patterns to actionable suggestions.
/// This is intentionally simple string matching for now — a real
/// implementation would match on token context, not message text,
/// but the parser doesn't expose enough structure for that yet.
pub fn suggest_fix(message: &str) -> Option<String> {
    if message.contains("Unknown statement starting with: END") {
        return Some(
            "Check for an unclosed block above this line (missing ENDIF, ENDFOR, or ENDWHILE)"
                .to_string(),
        );
    }

    if message.contains("Expected ENDIF") {
        return Some(
            "This IF block is missing its ENDIF. If you meant to chain conditions, use ELSEIF instead of ELSE IF"
                .to_string(),
        );
    }

    if message.contains("Expected ENDFOR") {
        return Some("This FOR block is missing its ENDFOR".to_string());
    }

    if message.contains("Expected ENDWHILE") {
        return Some("This WHILE block is missing its ENDWHILE".to_string());
    }

    if message.contains("Expected = for assignment") {
        return Some("Assignments use a single = (not == or :=)".to_string());
    }

    if message.contains("Expected identifier after INPUT") {
        return Some("INPUT must be followed by one or more variable names".to_string());
    }

    if message.starts_with("Unknown statement starting with:") {
        return Some(
            "This line doesn't match any known statement (INPUT, OUTPUT, IF, FOR, WHILE, DECLARE, or an assignment)"
                .to_string(),
        );
    }

    None
}
