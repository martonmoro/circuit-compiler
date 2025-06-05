mod ast;
mod circuit;
mod lexer;
mod optimizer;
mod parser;
mod ssa;
mod token;

use circuit::CircuitBuilder;
use lexer::Lexer;
use optimizer::ConstantFolder;
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

    let circuit_before = CircuitBuilder::from_ssa(ssa_program.clone());

    println!("\n=== CIRCUIT (BEFORE OPTIMIZATION) ===");
    println!("Public inputs: {:?}", circuit_before.public_inputs);
    println!("Private inputs: {:?}", circuit_before.private_inputs);
    for (i, gate) in circuit_before.gates.iter().enumerate() {
        println!("{}: {}", i, gate);
    }
    println!("output: {}", circuit_before.output_wire);
    println!("Total gates: {}", circuit_before.gates.len());

    let optimized_ssa = ConstantFolder::optimize(ssa_program);

    println!("\n=== OPTIMIZED SSA ===");
    for (i, instr) in optimized_ssa.instructions.iter().enumerate() {
        println!("{}: {}", i, instr);
    }
    println!("return {}", optimized_ssa.return_value);

    let circuit_after = CircuitBuilder::from_ssa(optimized_ssa);

    println!("\n=== CIRCUIT (AFTER OPTIMIZATION) ===");
    println!("Public inputs: {:?}", circuit_after.public_inputs);
    println!("Private inputs: {:?}", circuit_after.private_inputs);
    for (i, gate) in circuit_after.gates.iter().enumerate() {
        println!("{}: {}", i, gate);
    }
    println!("output: {}", circuit_after.output_wire);
    println!("Total gates: {}", circuit_after.gates.len());

    println!("\n=== OPTIMIZATION IMPACT ===");
    println!("Gates before: {}", circuit_before.gates.len());
    println!("Gates after: {}", circuit_after.gates.len());

    std::fs::create_dir_all("circuit").unwrap_or(()); // Create target dir if it doesn't exist

    let base_name = std::path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let output_filename = format!("circuit/{}.json", base_name);

    match circuit_after.save_to_file(&output_filename) {
        Ok(()) => println!("\nSaved circuit to {}", output_filename),
        Err(err) => eprintln!("Error saving: {}", err),
    }
}
