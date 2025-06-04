use crate::ast::{Expr, Program, Stmt};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SsaProgram {
    pub instructions: Vec<SsaInstruction>,
    pub return_value: SsaValue,
    pub public_inputs: Vec<SsaValue>,
    pub private_inputs: Vec<SsaValue>,
}

#[derive(Debug, Clone)]
pub enum SsaInstruction {
    Const(SsaValue, i32),              // destiantion, value
    Add(SsaValue, SsaValue, SsaValue), // destination, left, right
    Mul(SsaValue, SsaValue, SsaValue), // destination, left, right
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SsaValue {
    pub name: String,
    pub version: usize,
}

pub struct SsaBuilder {
    instructions: Vec<SsaInstruction>,
    var_versions: HashMap<String, usize>,
    temp_counter: usize,
    public_inputs: Vec<SsaValue>,
    private_inputs: Vec<SsaValue>,
}

impl SsaBuilder {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            var_versions: HashMap::new(),
            temp_counter: 0,
            public_inputs: Vec::new(),
            private_inputs: Vec::new(),
        }
    }

    pub fn convert(mut self, program: Program) -> SsaProgram {
        let mut return_value = None;

        for stmt in program.statements {
            match stmt {
                Stmt::PublicInput { name } => {
                    let version = self.next_variable_version(&name);
                    let input_ssa = SsaValue { name, version };
                    self.public_inputs.push(input_ssa);
                }
                Stmt::PrivateInput { name } => {
                    let version = self.next_variable_version(&name);
                    let input_ssa = SsaValue { name, version };
                    self.private_inputs.push(input_ssa);
                }
                Stmt::ConstDecl { name, value } => {
                    let temp = self.new_temp();
                    self.instructions
                        .push(SsaInstruction::Const(temp.clone(), value));

                    let version = self.next_variable_version(&name);
                    let var_ssa = SsaValue {
                        name: name.clone(),
                        version,
                    };

                    if let Some(last_instr) = self.instructions.pop() {
                        let new_instr = match last_instr {
                            SsaInstruction::Const(_, val) => SsaInstruction::Const(var_ssa, val),
                            _ => unreachable!(),
                        };
                        self.instructions.push(new_instr);
                    }
                }
                Stmt::Let { name, expr } => {
                    let _expr_result = self.convert_expr(expr);

                    let version = self.next_variable_version(&name);
                    let var_ssa = SsaValue {
                        name: name.clone(),
                        version,
                    };

                    // replace the destination of the last instruction
                    if let Some(last_instr) = self.instructions.pop() {
                        let new_instr = match last_instr {
                            SsaInstruction::Const(_, value) => {
                                SsaInstruction::Const(var_ssa, value)
                            }
                            SsaInstruction::Add(_, left, right) => {
                                SsaInstruction::Add(var_ssa, left, right)
                            }
                            SsaInstruction::Mul(_, left, right) => {
                                SsaInstruction::Mul(var_ssa, left, right)
                            }
                        };
                        self.instructions.push(new_instr);
                    }
                }
                Stmt::Return(expr) => {
                    return_value = Some(self.convert_expr(expr));
                }
            }
        }

        SsaProgram {
            instructions: self.instructions,
            return_value: return_value.expect("Program must have a return statement"),
            public_inputs: self.public_inputs,
            private_inputs: self.private_inputs,
        }
    }

    fn convert_expr(&mut self, expr: Expr) -> SsaValue {
        match expr {
            Expr::Literal(n) => {
                let temp = self.new_temp();
                self.instructions
                    .push(SsaInstruction::Const(temp.clone(), n));
                temp
            }
            // no instruction generated, just reading value
            Expr::Var(name) => {
                let current_version = self.var_versions.get(&name).copied().unwrap_or(0);
                SsaValue {
                    name,
                    version: current_version,
                }
            }
            Expr::Add(left, right) => {
                let left_val = self.convert_expr(*left);
                let right_val = self.convert_expr(*right);
                let result = self.new_temp();
                self.instructions
                    .push(SsaInstruction::Add(result.clone(), left_val, right_val));
                result
            }
            Expr::Mul(left, right) => {
                let left_val = self.convert_expr(*left);
                let right_val = self.convert_expr(*right);
                let result = self.new_temp();
                self.instructions
                    .push(SsaInstruction::Mul(result.clone(), left_val, right_val));
                result
            }
        }
    }
}

impl SsaBuilder {
    fn next_variable_version(&mut self, name: &str) -> usize {
        *self
            .var_versions
            .entry(name.to_string())
            .and_modify(|v| *v += 1)
            .or_insert(1)
    }

    fn new_temp(&mut self) -> SsaValue {
        let temp_name = format!("t{}", self.temp_counter); // "t0", "t1", "t2"
        let new_temp = SsaValue {
            name: temp_name,
            version: 0, // All temps get version 0
        };
        self.temp_counter += 1;
        new_temp
    }
}

impl std::fmt::Display for SsaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.name, self.version)
    }
}

impl std::fmt::Display for SsaInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SsaInstruction::Const(dest, value) => write!(f, "{} = {}", dest, value),
            SsaInstruction::Add(dest, left, right) => write!(f, "{} = {} + {}", dest, left, right),
            SsaInstruction::Mul(dest, left, right) => write!(f, "{} = {} * {}", dest, left, right),
        }
    }
}
