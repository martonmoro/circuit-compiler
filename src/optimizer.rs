use crate::ssa::{SsaInstruction, SsaProgram, SsaValue};
use std::collections::HashMap;

pub struct ConstantFolder {
    constants: HashMap<SsaValue, i32>,
}

impl ConstantFolder {
    pub fn new() -> Self {
        Self {
            constants: HashMap::new(),
        }
    }

    pub fn optimize(ssa_program: SsaProgram) -> SsaProgram {
        let mut folder = ConstantFolder::new();
        let mut optimized_instructions = Vec::new();

        for instr in &ssa_program.instructions {
            let folded_instr = folder.try_fold_instruction(instr);
            optimized_instructions.push(folded_instr);
        }

        SsaProgram {
            instructions: optimized_instructions,
            return_value: ssa_program.return_value,
            public_inputs: ssa_program.public_inputs,
            private_inputs: ssa_program.private_inputs,
        }
    }
}

impl ConstantFolder {
    fn get_constant_value(&self, ssa_value: &SsaValue) -> Option<i32> {
        self.constants.get(ssa_value).copied()
    }

    fn record_constant(&mut self, ssa_value: SsaValue, value: i32) {
        self.constants.insert(ssa_value, value);
    }

    fn try_fold_instruction(&mut self, instr: &SsaInstruction) -> SsaInstruction {
        match instr {
            SsaInstruction::Const(dest, value) => {
                self.record_constant(dest.clone(), *value);
                instr.clone()
            }
            SsaInstruction::Add(dest, left, right) => {
                if let (Some(left_val), Some(right_val)) = (
                    self.get_constant_value(left),
                    self.get_constant_value(right),
                ) {
                    let result = left_val + right_val;
                    self.record_constant(dest.clone(), result);

                    SsaInstruction::Const(dest.clone(), result)
                } else {
                    instr.clone()
                }
            }
            SsaInstruction::Mul(dest, left, right) => {
                if let (Some(left_val), Some(right_val)) = (
                    self.get_constant_value(left),
                    self.get_constant_value(right),
                ) {
                    let result = left_val * right_val;
                    self.record_constant(dest.clone(), result);

                    SsaInstruction::Const(dest.clone(), result)
                } else {
                    instr.clone()
                }
            }
        }
    }
}

pub struct DeadCodeEliminator;

impl DeadCodeEliminator {
    pub fn eliminate(ssa_program: SsaProgram) -> SsaProgram {
        let mut used_values = std::collections::HashSet::new();
        let mut input_dependent = std::collections::HashSet::new();

        // all inputs are used and input-dependent
        for input in &ssa_program.public_inputs {
            used_values.insert(input.clone());
            input_dependent.insert(input.clone());
        }
        for input in &ssa_program.private_inputs {
            used_values.insert(input.clone());
            input_dependent.insert(input.clone());
        }

        // return value is always used
        used_values.insert(ssa_program.return_value.clone());

        // all values that transitively depend on inputs
        let mut changed = true;
        while changed {
            changed = false;
            for instr in &ssa_program.instructions {
                let dest = Self::get_destination(instr);
                let inputs = Self::get_inputs(instr);

                // if any input to this instruction depends on circuit inputs,
                // then this instruction's output also depends on circuit inputs
                if inputs.iter().any(|input| input_dependent.contains(input)) {
                    if input_dependent.insert(dest.clone()) {
                        changed = true;
                    }
                }
            }
        }

        // all input-dependent values are used
        for value in &input_dependent {
            used_values.insert(value.clone());
        }

        // backwards reachability from used values
        changed = true;
        while changed {
            changed = false;
            for instr in &ssa_program.instructions {
                let dest = Self::get_destination(instr);
                if used_values.contains(&dest) {
                    for input in Self::get_inputs(instr) {
                        if used_values.insert(input) {
                            changed = true;
                        }
                    }
                }
            }
        }

        let filtered_instructions: Vec<_> = ssa_program
            .instructions
            .into_iter()
            .filter(|instr| {
                let dest = Self::get_destination(instr);
                used_values.contains(&dest)
            })
            .collect();

        SsaProgram {
            instructions: filtered_instructions,
            return_value: ssa_program.return_value,
            public_inputs: ssa_program.public_inputs,
            private_inputs: ssa_program.private_inputs,
        }
    }

    fn get_destination(instr: &SsaInstruction) -> SsaValue {
        match instr {
            SsaInstruction::Const(dest, _) => dest.clone(),
            SsaInstruction::Add(dest, _, _) => dest.clone(),
            SsaInstruction::Mul(dest, _, _) => dest.clone(),
        }
    }

    fn get_inputs(instr: &SsaInstruction) -> Vec<SsaValue> {
        match instr {
            SsaInstruction::Const(_, _) => vec![],
            SsaInstruction::Add(_, left, right) => vec![left.clone(), right.clone()],
            SsaInstruction::Mul(_, left, right) => vec![left.clone(), right.clone()],
        }
    }
}
