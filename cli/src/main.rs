use pseudocode_core::formatter::Formatter;
use pseudocode_core::interpreter::Interpreter;
use pseudocode_core::lexer::Lexer;
use pseudocode_core::parser::Parser;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args
        .first()
        .map(|s| s.as_str())
        .unwrap_or("pseudocode-interpreter");

    if args.len() < 2 {
        print_usage(program_name);
        return;
    }

    let filename = &args[1];

    // Check for formatting flag
    let fmt_mode = args.len() > 2 && args[2] == "--fmt";
    let debug_mode = args.len() > 2 && args[2] == "--debug";

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            return;
        }
    };

    // If --fmt flag is present, format and output
    if fmt_mode {
        let diagnostics = pseudocode_checker::check(&source);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == pseudocode_checker::Severity::Error)
            .collect();

        if !errors.is_empty() {
            eprintln!("Cannot format {}: syntax errors found", filename);
            for err in &errors {
                eprintln!(
                    "  error at line {}, column {}: {}",
                    err.line, err.column, err.message
                );
                if let Some(suggestion) = &err.suggestion {
                    eprintln!("    suggestion: {}", suggestion);
                }
            }
            std::process::exit(1);
        }

        let warnings: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == pseudocode_checker::Severity::Warning)
            .collect();
        for warn in &warnings {
            eprintln!(
                "warning at line {}, column {}: {}",
                warn.line, warn.column, warn.message
            );
        }

        let mut formatter = Formatter::new();
        match formatter.format(&source) {
            Ok(formatted) => {
                if let Err(e) = fs::write(filename, formatted) {
                    eprintln!("Error writing formatted file: {}", e);
                } else {
                    println!("✓ Formatted: {}", filename);
                }
            }
            Err(e) => {
                eprintln!("Formatting error: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    let mut lexer = Lexer::new(source.clone());
    let tokens = lexer.tokenize();

    if debug_mode {
        println!("--- Tokens ---");
        for (i, token) in tokens.iter().enumerate() {
            println!("{:3}: {:?}", i, token);
        }
        println!();
    }
    let mut parser = Parser::new(tokens);
    let (ast, parse_errors) = parser.parse();

    for err in &parse_errors {
        eprintln!("Parse error: {}", err);
    }
    if debug_mode {
        println!("--- Abstract Syntax Tree ---");
        println!("{:#?}", ast);
        println!();
    }

    let mut interpreter = Interpreter::new();

    match interpreter.run(&ast) {
        Ok(_) => {
            if debug_mode {
                println!();
                interpreter.print_state();
            }
        }
        Err(e) => {
            eprintln!();
            eprintln!("Runtime Error: {}", e);
        }
    }
}

fn print_usage(program_name: &str) {
    println!("Pseudocode Interpreter");
    println!();
    println!("Usage:");
    println!(
        "  {} <filename.psc>          Run a pseudocode file",
        program_name
    );
    println!(
        "  {} <filename.psc> --debug  Run with debug output",
        program_name
    );
    println!(
        "  {} <filename.psc> --fmt    Format a pseudocode file",
        program_name
    );
    println!();
    println!("Examples:");
    println!("  {} program.psc", program_name);
    println!("  {} program.psc --debug", program_name);
    println!("  {} program.psc --fmt", program_name);
}
