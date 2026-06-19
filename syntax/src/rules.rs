/// Maps known parse-error message patterns to actionable suggestions.
/// This is intentionally simple string matching for now — a real
/// implementation would match on token context, not message text,
/// but the parser doesn't expose enough structure for that yet.
pub fn suggest_fix(message: &str) -> Option<String> {
    if message.contains("Unknown statement starting with: END") {
        return Some(
            "Check for an unclosed block above this line (missing ENDIF, ENDFOR, ENDWHILE, or ENDFUNCTION)"
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

    if message.contains("Expected ENDFUNCTION") {
        return Some("This FUNCTION block is missing its ENDFUNCTION".to_string());
    }

    if message.contains("Expected function name after FUNCTION") {
        return Some("FUNCTION must be followed by a name, e.g. FUNCTION sum(n)".to_string());
    }

    if message.contains("Expected ( after function name") {
        return Some(
            "Function declarations need a parameter list in parentheses, even if empty: FUNCTION name()"
                .to_string(),
        );
    }

    if message.contains("Expected parameter name") {
        return Some(
            "Each item in a function's parameter list must be a plain identifier, separated by commas"
                .to_string(),
        );
    }

    if message.contains("Expected module name after IMPORT") {
        return Some(
            "IMPORT must be followed by a module name, e.g. IMPORT _MATH or IMPORT _MATH[SIN, COS]"
                .to_string(),
        );
    }

    if message.contains("Expected function name in import list") {
        return Some(
            "Names inside an IMPORT's brackets must be plain identifiers separated by commas, e.g. IMPORT _MATH[SIN, COS]"
                .to_string(),
        );
    }

    if message.contains("Program should start with START") {
        return Some(
            "IMPORT statements are only valid before START — check that all imports come before the START keyword"
                .to_string(),
        );
    }

    if message.contains("Expected identifier after CONST") {
        return Some("CONST must be followed by a name, e.g. CONST MAX = 100".to_string());
    }

    if message.contains("Expected = after CONST name") {
        return Some(
            "CONST declarations must assign a value immediately, e.g. CONST MAX = 100".to_string(),
        );
    }

    if message.contains("Expected identifier after STATIC") {
        return Some("STATIC must be followed by a name, e.g. STATIC count = 0".to_string());
    }

    if message.contains("Expected = after STATIC name") {
        return Some(
            "STATIC declarations must assign a starting value immediately, e.g. STATIC count = 0"
                .to_string(),
        );
    }

    if message.contains("Cannot reassign constant") {
        return Some(
            "This name was declared with CONST and cannot be reassigned. Use a regular variable if it needs to change"
                .to_string(),
        );
    }

    if message.contains("Constant already declared") {
        return Some("This CONST name was already declared earlier; choose a different name, or remove the duplicate declaration".to_string());
    }

    if message.contains("Expected = for assignment") {
        return Some("Assignments use a single = (not == or :=)".to_string());
    }

    if message.contains("Expected identifier after INPUT") {
        return Some("INPUT must be followed by one or more variable names".to_string());
    }

    if message.starts_with("Unknown statement starting with:") {
        return Some(
            "This line doesn't match any known statement (INPUT, OUTPUT, IF, FOR, WHILE, DECLARE, FUNCTION, RETURN, CONST, STATIC, or an assignment/function call)"
                .to_string(),
        );
    }

    if message.starts_with("Unexpected token:") {
        return Some(
        "This token appears somewhere an expression or value was expected — check for a missing operand, misplaced keyword, or unclosed block above this line"
            .to_string(),
    );
    }

    if message.contains("Expected , or ) in parameter list") {
        return Some(
            "Function parameters must be separated by commas, e.g. FUNCTION add(a, b)".to_string(),
        );
    }

    None
}
