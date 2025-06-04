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
