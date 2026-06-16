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
        println!("Pseudocode Interpreter");
        println!();
        println!("Usage:");
        println!("  {} <filename.psc>", program_name);
        println!();
        return;
    }

    let filename = &args[1];
    let debug_mode = args.len() > 2 && args[2] == "--debug";

    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", filename, e);
            return;
        }
    };

    println!("=== Running: {} ===", filename);
    println!();

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    if debug_mode {
        println!("--- Tokens ---");
        for (i, token) in tokens.iter().enumerate() {
            println!("{:3}: {:?}", i, token);
        }
        println!();
    }

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    if debug_mode {
        println!("--- Abstract Syntax Tree ---");
        println!("{:#?}", ast);
        println!();
    }

    let mut interpreter = Interpreter::new();

    match interpreter.run(&ast) {
        Ok(_) => {
            println!();
            println!("=== Execution Complete ===");

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
