use pseudocode_core::formatter::Formatter;
use pseudocode_core::interpreter::Interpreter;
use pseudocode_core::lexer::Lexer;
use pseudocode_core::parser::Parser;
use std::env;
use std::fs;
use std::path::Path;

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

    // Resolve and load sibling .psc files' PUB exports before running
    // the entry file itself.
    let mut interpreter = Interpreter::new();
    match load_sibling_exports(filename) {
        Ok(exports) => {
            if let Err(e) = interpreter.merge_exports(exports) {
                eprintln!("Error merging sibling exports: {}", e);
                return;
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
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

/// Scans the entry file's directory for sibling .psc files, runs each
/// one in full isolation, collects their PUB exports, and returns the
/// merged set — erroring hard if any sibling fails to parse, or if two
/// siblings export a name that collides.
fn load_sibling_exports(
    entry_filename: &str,
) -> Result<Vec<pseudocode_core::interpreter::Export>, String> {
    let entry_path = Path::new(entry_filename);
    let dir = match entry_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let entry_canonical = fs::canonicalize(entry_path).ok();

    let mut sibling_paths = Vec::new();
    let read_dir = fs::read_dir(dir).map_err(|e| format!("Error reading directory: {}", e))?;
    for entry in read_dir {
        let entry = entry.map_err(|e| format!("Error reading directory entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("psc") {
            continue;
        }
        if let Some(canonical) = &entry_canonical {
            if fs::canonicalize(&path).ok().as_ref() == Some(canonical) {
                continue; // Skip the entry file itself.
            }
        }
        sibling_paths.push(path);
    }
    sibling_paths.sort();

    let mut all_exports: Vec<pseudocode_core::interpreter::Export> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for path in sibling_paths {
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("Error reading {}: {}", path.display(), e))?;

        let diagnostics = pseudocode_checker::check(&source);
        let has_errors = diagnostics
            .iter()
            .any(|d| d.severity == pseudocode_checker::Severity::Error);
        if has_errors {
            let mut msg = format!(
                "Cannot load sibling module {}: syntax errors found\n",
                path.display()
            );
            for d in diagnostics
                .iter()
                .filter(|d| d.severity == pseudocode_checker::Severity::Error)
            {
                msg.push_str(&format!(
                    "  error at line {}, column {}: {}\n",
                    d.line, d.column, d.message
                ));
            }
            return Err(msg);
        }

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _parse_errors) = parser.parse();

        let mut sibling_interpreter = Interpreter::new();
        sibling_interpreter
            .run(&ast)
            .map_err(|e| format!("Runtime error in sibling module {}: {}", path.display(), e))?;

        let exports = sibling_interpreter
            .collect_exports(&ast)
            .map_err(|e| format!("Error collecting exports from {}: {}", path.display(), e))?;

        for export in &exports {
            let name = export_name(export);
            if !seen_names.insert(name.clone()) {
                return Err(format!(
                    "Ambiguous PUB export: '{}' is declared in more than one sibling file",
                    name
                ));
            }
        }

        all_exports.extend(exports);
    }

    Ok(all_exports)
}

fn export_name(export: &pseudocode_core::interpreter::Export) -> String {
    match export {
        pseudocode_core::interpreter::Export::Function { name, .. } => name.clone(),
        pseudocode_core::interpreter::Export::Const { name, .. } => name.clone(),
        pseudocode_core::interpreter::Export::Array { name, .. } => name.clone(),
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
