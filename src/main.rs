mod ast;
mod circuit;
mod lexer;
mod optimizer;
mod parser;
mod ssa;
mod token;
mod witness;

use circuit::CircuitBuilder;
use lexer::Lexer;
use optimizer::{ConstantFolder, DeadCodeEliminator};
use parser::Parser;
use ssa::SsaBuilder;
use std::env;
use std::fs;
use std::process;
use witness::{InputFile, WitnessCalculator};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage:");
        eprintln!("  cargo run <file.zk>              # Compile only");
        eprintln!("  cargo run <file.zk> <inputs.toml> # Compile and execute");
        process::exit(1);
    }

    let filename = &args[1];
    let inputs_filename = args.get(2);

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

    let folded_ssa = ConstantFolder::optimize(ssa_program.clone());
    let optimized_ssa = DeadCodeEliminator::eliminate(folded_ssa);

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

    fs::create_dir_all("circuit").unwrap_or(());
    let base_name = std::path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();
    let circuit_filename = format!("circuit/{}.json", base_name);

    match circuit_after.save_to_file(&circuit_filename) {
        Ok(()) => println!("\nSaved circuit to {}", circuit_filename),
        Err(err) => {
            eprintln!("Error saving circuit: {}", err);
            process::exit(1);
        }
    }

    let r1cs = circuit_after.to_r1cs();
    let r1cs_filename = format!("circuit/{}.r1cs", base_name);
    match r1cs.save_to_file(&r1cs_filename) {
        Ok(()) => println!("Saved R1CS to {}", r1cs_filename),
        Err(err) => eprintln!("Error saving R1CS: {}", err),
    }

    if let Some(inputs_file) = inputs_filename {
        println!("\n=== CALCULATING WITNESS ===");

        let inputs_content = match fs::read_to_string(inputs_file) {
            Ok(content) => content,
            Err(err) => {
                eprintln!("Error reading inputs file: {}", err);
                process::exit(1);
            }
        };

        let inputs: InputFile = match toml::from_str(&inputs_content) {
            Ok(inputs) => inputs,
            Err(err) => {
                eprintln!("Error parsing inputs file: {}", err);
                process::exit(1);
            }
        };

        let mut calculator = WitnessCalculator::new();
        match calculator.calculate_witness(&circuit_after, inputs) {
            Ok(result) => {
                println!("Witness calculation complete");
                println!("Result: {}", result);

                let witness_filename = format!("circuit/{}.witness", base_name);
                match calculator.save_r1cs_witness(&circuit_after, &witness_filename, result) {
                    Ok(()) => println!("Saved witness to {}", witness_filename),
                    Err(err) => eprintln!("Error saving witness: {}", err),
                }
            }
            Err(err) => {
                eprintln!("Witness calculation error: {}", err);
                process::exit(1);
            }
        }
    }
}
