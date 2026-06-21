use psycore::formatter::Formatter;
use psycore::interpreter::Interpreter;
use psycore::lexer::Lexer;
use psycore::parser::Parser;
use psycore::parser::ast::Statement;
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program_name = args.first().map(|s| s.as_str()).unwrap_or("interpreter");

    if args.len() < 2 {
        print_usage(program_name);
        return;
    }

    let filename = &args[1];

    let fmt_mode = args.len() > 2 && args[2] == "--fmt";
    let debug_mode = args.len() > 2 && args[2] == "--debug";

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            return;
        }
    };

    if fmt_mode {
        let diagnostics = syntax::check(&source);
        let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.severity == syntax::Severity::Error)
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
            .filter(|d| d.severity == syntax::Severity::Warning)
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
        for (i, token) in tokens.iter().enumerate() {
            println!("{:3}: {:?}", i, token);
        }
    }

    let mut parser = Parser::new(tokens);
    let (ast, parse_errors) = parser.parse();

    for err in &parse_errors {
        eprintln!("Parse error: {}", err);
    }

    if debug_mode {
        println!("{:#?}", ast);
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

/// Returns true if the AST contains at least one PUB declaration.
/// Only files with PUB exports are executed as sibling modules.
fn has_pub_exports(ast: &[psycore::parser::ast::Spanned<Statement>]) -> bool {
    ast.iter()
        .any(|stmt| matches!(&stmt.node, Statement::Public(_)))
}

/// Scans the entry file's directory for sibling .psy files that contain
/// PUB declarations, runs only those files to collect their exports,
/// and returns the merged set.
///
/// Files with no PUB declarations are skipped entirely — they are plain
/// scripts and running them as modules would cause unwanted side effects.
fn load_sibling_exports(entry_filename: &str) -> Result<Vec<psycore::interpreter::Export>, String> {
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
        if path.extension().and_then(|e| e.to_str()) != Some("psy") {
            continue;
        }
        if let Some(canonical) = &entry_canonical {
            if fs::canonicalize(&path).ok().as_ref() == Some(canonical) {
                continue; // skip entry file itself
            }
        }
        sibling_paths.push(path);
    }
    sibling_paths.sort();

    let mut all_exports: Vec<psycore::interpreter::Export> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for path in sibling_paths {
        let source = fs::read_to_string(&path)
            .map_err(|e| format!("Error reading {}: {}", path.display(), e))?;

        // Parse first to check for PUB exports — skip syntax check
        // entirely for files that aren't modules.
        let mut lexer = Lexer::new(source.clone());
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (ast, _parse_errors) = parser.parse();

        // Skip files with no PUB declarations — they are standalone
        // scripts and should never be executed as side-effect modules.
        if !has_pub_exports(&ast) {
            continue;
        }

        // Only syntax-check files that actually export something.
        let diagnostics = syntax::check(&source);
        let has_errors = diagnostics
            .iter()
            .any(|d| d.severity == syntax::Severity::Error);
        if has_errors {
            let mut msg = format!(
                "Cannot load sibling module {}: syntax errors found\n",
                path.display()
            );
            for d in diagnostics
                .iter()
                .filter(|d| d.severity == syntax::Severity::Error)
            {
                msg.push_str(&format!(
                    "  error at line {}, column {}: {}\n",
                    d.line, d.column, d.message
                ));
            }
            return Err(msg);
        }

        let mut sibling_interpreter = Interpreter::new();
        sibling_interpreter
            .run_exports_only(&ast)
            .map_err(|e| format!("Error loading exports from {}: {}", path.display(), e))?;

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

fn export_name(export: &psycore::interpreter::Export) -> String {
    match export {
        psycore::interpreter::Export::Function { name, .. } => name.clone(),
        psycore::interpreter::Export::Const { name, .. } => name.clone(),
        psycore::interpreter::Export::Array { name, .. } => name.clone(),
    }
}

fn print_usage(program_name: &str) {
    println!("File input not provided for {}", program_name);
}
