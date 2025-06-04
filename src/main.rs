mod ast;
mod lexer;
mod parser;
mod ssa;
mod token;

use lexer::Lexer;
use parser::Parser;
use ssa::SsaBuilder;
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

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("\n=== TOKENS ===");
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }

    let mut parser = Parser::new(tokens);
    let program = match parser.parse() {
        Ok(program) => {
            println!("\n=== AST ===");
            println!("{:#?}", program);
            program
        }
        Err(err) => {
            eprintln!("\n=== PARSE ERROR ===");
            eprintln!("{}", err.message);
            process::exit(1);
        }
    };

    let ssa_builder = SsaBuilder::new();
    let ssa_program = ssa_builder.convert(program);

    println!("\n=== SSA IR ===");
    for (i, instr) in ssa_program.instructions.iter().enumerate() {
        println!("{}: {}", i, instr);
    }
    println!("return {}", ssa_program.return_value);
}
