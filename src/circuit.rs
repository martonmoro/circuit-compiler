use std::collections::HashMap;

use crate::ssa::{SsaInstruction, SsaProgram, SsaValue};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Wire {
    pub id: usize,
}

#[derive(Debug, Clone)]
pub enum Gate {
    Const {
        output: Wire,
        value: i32,
    },
    Add {
        output: Wire,
        left: Wire,
        right: Wire,
    },
    Mul {
        output: Wire,
        left: Wire,
        right: Wire,
    },
}

#[derive(Debug, Clone)]
pub struct Circuit {
    pub gates: Vec<Gate>,
    pub output_wire: Wire,
}

pub struct CircuitBuilder {
    gates: Vec<Gate>,
    wire_counter: usize,
    ssa_to_wire: HashMap<SsaValue, Wire>,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        Self {
            gates: Vec::new(),
            wire_counter: 0,
            ssa_to_wire: HashMap::new(),
        }
    }

    pub fn from_ssa(ssa_program: SsaProgram) -> Circuit {
        let mut builder = CircuitBuilder::new();

        for instr in &ssa_program.instructions {
            builder.convert_instruction(instr);
        }

        let output_wire = builder.get_or_create_wire(&ssa_program.return_value);

        Circuit {
            gates: builder.gates,
            output_wire,
        }
    }
}

impl CircuitBuilder {
    fn new_wire(&mut self) -> Wire {
        let wire = Wire {
            id: self.wire_counter,
        };
        self.wire_counter += 1;
        wire
    }

    fn get_or_create_wire(&mut self, ssa_value: &SsaValue) -> Wire {
        if let Some(wire) = self.ssa_to_wire.get(ssa_value) {
            wire.clone()
        } else {
            let new_wire = self.new_wire();
            self.ssa_to_wire.insert(ssa_value.clone(), new_wire.clone());
            new_wire
        }
    }

    fn convert_instruction(&mut self, instr: &SsaInstruction) -> Wire {
        match instr {
            SsaInstruction::Const(dest, value) => {
                let dest_wire = self.get_or_create_wire(dest);
                let gate = Gate::Const {
                    output: dest_wire.clone(),
                    value: *value,
                };
                self.gates.push(gate);
                dest_wire
            }
            SsaInstruction::Add(dest, left, right) => {
                let dest_wire = self.get_or_create_wire(dest);
                let left_wire = self.get_or_create_wire(left);
                let right_wire = self.get_or_create_wire(right);
                let gate = Gate::Add {
                    output: dest_wire.clone(),
                    left: left_wire,
                    right: right_wire,
                };
                self.gates.push(gate);
                dest_wire
            }
            SsaInstruction::Mul(dest, left, right) => {
                let dest_wire = self.get_or_create_wire(dest);
                let left_wire = self.get_or_create_wire(left);
                let right_wire = self.get_or_create_wire(right);
                let gate = Gate::Mul {
                    output: dest_wire.clone(),
                    left: left_wire,
                    right: right_wire,
                };
                self.gates.push(gate);
                dest_wire
            }
        }
    }
}

impl std::fmt::Display for Wire {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "w{}", self.id)
    }
}

impl std::fmt::Display for Gate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Gate::Const { output, value } => write!(f, "{} = {}", output, value),
            Gate::Add {
                output,
                left,
                right,
            } => write!(f, "{} = {} + {}", output, left, right),
            Gate::Mul {
                output,
                left,
                right,
            } => write!(f, "{} = {} * {}", output, left, right),
        }
    }
}
