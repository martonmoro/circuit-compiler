use crate::ssa::{SsaInstruction, SsaProgram, SsaValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wire {
    pub id: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Assert {
        output: Wire,
        left: Wire,
        right: Wire,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circuit {
    pub public_inputs: Vec<(String, Wire)>,
    pub private_inputs: Vec<(String, Wire)>,
    pub gates: Vec<Gate>,
    pub output_wire: Wire,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R1csConstraint {
    pub a: Vec<i32>,
    pub b: Vec<i32>,
    pub c: Vec<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R1csSystem {
    pub num_constraints: usize,
    pub num_variables: usize,
    pub constraints: Vec<R1csConstraint>,
    pub public_inputs: Vec<(String, usize)>,
    pub private_inputs: Vec<(String, usize)>,
    pub output_wire: usize,
}

impl R1csSystem {
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }
}

pub struct CircuitBuilder {
    gates: Vec<Gate>,
    wire_counter: usize,
    ssa_to_wire: HashMap<SsaValue, Wire>,
    public_inputs: Vec<(String, Wire)>,
    private_inputs: Vec<(String, Wire)>,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        Self {
            gates: Vec::new(),
            wire_counter: 0,
            ssa_to_wire: HashMap::new(),
            public_inputs: Vec::new(),
            private_inputs: Vec::new(),
        }
    }

    pub fn from_ssa(ssa_program: SsaProgram) -> Circuit {
        let mut builder = CircuitBuilder::new();

        for input in &ssa_program.public_inputs {
            let wire = builder.get_or_create_wire(input);
            let name = input.name.clone();
            builder.public_inputs.push((name, wire));
        }

        for input in &ssa_program.private_inputs {
            let wire = builder.get_or_create_wire(input);
            let name = input.name.clone();
            builder.private_inputs.push((name, wire));
        }

        for instr in &ssa_program.instructions {
            builder.convert_instruction(instr);
        }

        let output_wire = builder.get_or_create_wire(&ssa_program.return_value);

        Circuit {
            public_inputs: builder.public_inputs,
            private_inputs: builder.private_inputs,
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
            SsaInstruction::Assert(left, right) => {
                let left_wire = self.get_or_create_wire(left);
                let right_wire = self.get_or_create_wire(right);
                let zero_wire = self.new_wire();
                let gate = Gate::Assert {
                    output: zero_wire.clone(),
                    left: left_wire,
                    right: right_wire,
                };
                self.gates.push(gate);
                zero_wire
            }
        }
    }
}

impl Circuit {
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    pub fn to_r1cs(&self) -> R1csSystem {
        // get max wire id + 1
        let num_wires = self
            .gates
            .iter()
            .flat_map(|gate| match gate {
                Gate::Const { output, .. } => vec![output.id],
                Gate::Add {
                    output,
                    left,
                    right,
                } => vec![output.id, left.id, right.id],
                Gate::Mul {
                    output,
                    left,
                    right,
                } => vec![output.id, left.id, right.id],
                Gate::Assert {
                    output,
                    left,
                    right,
                } => vec![output.id, left.id, right.id],
            })
            .chain(self.public_inputs.iter().map(|(_, wire)| wire.id))
            .chain(self.private_inputs.iter().map(|(_, wire)| wire.id))
            .chain(std::iter::once(self.output_wire.id))
            .max()
            .unwrap_or(0)
            + 1;

        let mut constraints = Vec::new();

        for gate in &self.gates {
            let constraint = match gate {
                Gate::Const { output, value } => {
                    // 0 * value = output
                    let mut a = vec![0; num_wires];
                    let mut b = vec![0; num_wires];
                    let mut c = vec![0; num_wires];

                    a[0] = 1;
                    b[0] = *value; // Constant term
                    c[output.id] = 1;

                    R1csConstraint { a, b, c }
                }
                Gate::Mul {
                    output,
                    left,
                    right,
                } => {
                    // left * right = output
                    let mut a = vec![0; num_wires];
                    let mut b = vec![0; num_wires];
                    let mut c = vec![0; num_wires];

                    a[left.id] = 1;
                    b[right.id] = 1;
                    c[output.id] = 1;

                    R1csConstraint { a, b, c }
                }
                Gate::Add {
                    output,
                    left,
                    right,
                } => {
                    // (left + right) * 1 = output
                    let mut a = vec![0; num_wires];
                    let mut b = vec![0; num_wires];
                    let mut c = vec![0; num_wires];

                    a[left.id] = 1;
                    a[right.id] = 1;
                    b[0] = 1; // multiply by 1
                    c[output.id] = 1;

                    R1csConstraint { a, b, c }
                }
                Gate::Assert {
                    output,
                    left,
                    right,
                } => {
                    // (left - right) * 1 = output (should be 0)
                    let mut a = vec![0; num_wires];
                    let mut b = vec![0; num_wires];
                    let mut c = vec![0; num_wires];

                    a[left.id] = 1;
                    a[right.id] = -1;
                    b[0] = 1; // multiply by 1
                    c[output.id] = 1;

                    R1csConstraint { a, b, c }
                }
            };
            constraints.push(constraint);
        }

        R1csSystem {
            num_constraints: constraints.len(),
            num_variables: num_wires,
            constraints,
            public_inputs: self
                .public_inputs
                .iter()
                .map(|(name, wire)| (name.clone(), wire.id))
                .collect(),
            private_inputs: self
                .private_inputs
                .iter()
                .map(|(name, wire)| (name.clone(), wire.id))
                .collect(),
            output_wire: self.output_wire.id,
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
            Gate::Assert {
                output,
                left,
                right,
            } => write!(f, "{} = {} - {}", output, left, right),
        }
    }
}
