# ZK Circuit Compiler

A zero-knowledge circuit compiler that transforms high-level programs into arithmetic circuits and R1CS constraint systems.

## Features

- High-level language with public/private inputs and assertions
- SSA intermediate representation
- Circuit optimization (constant folding, dead code elimination)
- R1CS constraint system generation
- Witness calculation and verification

## Grammar

```

program = statement*

statement = "public" IDENT
| "private" IDENT
| "const" IDENT "=" NUMBER
| "let" IDENT "=" expr
| "assert" expr "==" expr
| "return" expr

expr = term ("+" term | "_" term)_

term = IDENT | NUMBER | "(" expr ")"

```

## Usage

```bash
# Compile only
cargo run examples/simple.zk

# Compile and execute
cargo run examples/simple.zk inputs/inputs.toml
```

Generates:

- `circuit/simple.json` - Circuit gates
- `circuit/simple.r1cs` - R1CS constraints
- `circuit/simple.witness` - Execution trace

## Architecture

The compiler pipeline:

1. **Lexer** → tokens
2. **Parser** → AST
3. **SSA conversion** → intermediate form
4. **Optimization** → constant folding, dead code elimination
5. **Circuit generation** → arithmetic gates
6. **R1CS generation** → constraint matrix
7. **Witness calculation** → execution with inputs

## Current State

Uses `i32` arithmetic for simplicity. Production ZK requires finite field arithmetic but my aim with this project was to explore the different compiler techniques.

## Examples

Check `examples/` for test programs showing language features and optimizations. The current example files end with `.zk`, the files ending in `.mc` were used for early testing.
