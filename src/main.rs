mod interpreter;
mod lexer;
mod parser;

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Pseudocode Interpreter - COS102 Project");
        println!();
        println!("Usage:");
        println!("  cargo run -- <filename.psc> [--debug]");
        println!();
        println!("Examples:");
        println!("  cargo run -- examples/average.psc");
        println!("  cargo run -- examples/average.psc --debug");
        println!("  cargo run -- examples/debug_demo.psc --debug");
        return;
    }

    let filename = &args[1];
    let debug_mode = args.len() > 2 && args[2] == "--debug";

    // Read the pseudocode file
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            return;
        }
    };

    println!("=== Running: {} ===", filename);
    println!();

    // Step 1: Lexical analysis (tokenization)
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    // Print tokens in debug mode
    if debug_mode {
        println!("--- Tokens ---");
        for (i, token) in tokens.iter().enumerate() {
            println!("{:3}: {:?}", i, token);
        }
        println!();
    }

    // Step 2: Parsing
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    // Print AST in debug mode
    if debug_mode {
        println!("--- Abstract Syntax Tree ---");
        println!("{:#?}", ast);
        println!();
    }

    // Step 3: Interpretation
    let mut interpreter = Interpreter::new();

    match interpreter.run(&ast) {
        Ok(_) => {
            println!();
            println!("=== Execution Complete ===");

            // Show variable state in debug mode
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
