mod ast;
mod lexer;
mod parser; // Add this line
mod token;

use lexer::Lexer;
use parser::Parser; // Add this line
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: cargo run <input_file>");
        process::exit(1);
    }

    let filename = &args[1];
    let source = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file '{}': {}", filename, err);
            process::exit(1);
        }
    };

    println!("=== SOURCE ===");
    println!("{}", source);

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("\n=== TOKENS ===");
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }

    // Parse
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(program) => {
            println!("\n=== AST ===");
            println!("{:#?}", program);
        }
        Err(err) => {
            eprintln!("\n=== PARSE ERROR ===");
            eprintln!("{}", err.message);
            process::exit(1);
        }
    }
}
