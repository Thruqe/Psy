pub mod checker;
pub mod diagnostics;
pub mod rules;

pub use checker::check;
pub use diagnostics::{Diagnostic, Severity};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_grade_check_errors() {
        let source = r#"
START
    INPUT score
    IF score >= 70 THEN
        OUTPUT "Grade: A"
    ELSE IF score >= 60 THEN
        OUTPUT "Grade: B"
    ENDIF
END
"#;
        let diagnostics = check(source);
        assert!(!diagnostics.is_empty());
        for d in &diagnostics {
            println!("{:?}", d);
        }
    }
}
