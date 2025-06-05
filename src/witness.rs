use crate::circuit::{Circuit, Gate, Wire};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct InputFile {
    pub public: Option<HashMap<String, i32>>,
    pub private: Option<HashMap<String, i32>>,
}

#[derive(Debug)]
pub enum WitnessError {
    MissingPublicInput(String),
    MissingPrivateInput(String),
    MissingWireValue(String),
    NoPublicInputsProvided,
    NoPrivateInputsProvided,
}

impl std::fmt::Display for WitnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WitnessError::MissingPublicInput(name) => write!(f, "Missing public input: {}", name),
            WitnessError::MissingPrivateInput(name) => write!(f, "Missing private input: {}", name),
            WitnessError::MissingWireValue(wire) => write!(f, "Wire {} has no value", wire),
            WitnessError::NoPublicInputsProvided => {
                write!(f, "Circuit requires public inputs but none provided")
            }
            WitnessError::NoPrivateInputsProvided => {
                write!(f, "Circuit requires private inputs but none provided")
            }
        }
    }
}

impl std::error::Error for WitnessError {}

pub struct WitnessCalculator {
    wire_values: HashMap<Wire, i32>,
}

impl WitnessCalculator {
    pub fn new() -> Self {
        Self {
            wire_values: HashMap::new(),
        }
    }

    fn get_wire_value(&self, wire: &Wire) -> Option<i32> {
        self.wire_values.get(wire).copied()
    }

    pub fn calculate_witness(
        &mut self,
        circuit: &Circuit,
        inputs: InputFile,
    ) -> Result<i32, WitnessError> {
        self.set_inputs(circuit, inputs)?;

        for gate in &circuit.gates {
            self.execute_gate(gate)?;
        }

        self.get_wire_value(&circuit.output_wire)
            .ok_or_else(|| WitnessError::MissingWireValue(circuit.output_wire.to_string()))
    }

    pub fn save_r1cs_witness(
        &self,
        circuit: &Circuit,
        filename: &str,
        result: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use serde_json::json;

        let max_wire_id = self
            .wire_values
            .keys()
            .map(|wire| wire.id)
            .max()
            .unwrap_or(0);
        let mut witness = vec![0; max_wire_id + 1];

        for (wire, value) in &self.wire_values {
            witness[wire.id] = *value;
        }

        let mut public_inputs = HashMap::new();
        for (name, wire) in &circuit.public_inputs {
            public_inputs.insert(name, self.get_wire_value(wire).unwrap_or(0));
        }

        let mut private_inputs = HashMap::new();
        for (name, wire) in &circuit.private_inputs {
            private_inputs.insert(name, self.get_wire_value(wire).unwrap_or(0));
        }

        let witness_data = json!({
            "witness": witness,
            "public_inputs": public_inputs,
            "private_inputs": private_inputs,
            "result": result,
            "num_wires": witness.len()
        });

        std::fs::write(filename, serde_json::to_string_pretty(&witness_data)?)?;
        Ok(())
    }

    fn set_inputs(&mut self, circuit: &Circuit, inputs: InputFile) -> Result<(), WitnessError> {
        if let Some(public_vals) = inputs.public {
            for (name, wire) in &circuit.public_inputs {
                if let Some(value) = public_vals.get(name) {
                    self.wire_values.insert(wire.clone(), *value);
                } else {
                    return Err(WitnessError::MissingPublicInput(name.clone()));
                }
            }
        } else if !circuit.public_inputs.is_empty() {
            return Err(WitnessError::NoPublicInputsProvided);
        }

        if let Some(private_vals) = inputs.private {
            for (name, wire) in &circuit.private_inputs {
                if let Some(value) = private_vals.get(name) {
                    self.wire_values.insert(wire.clone(), *value);
                } else {
                    return Err(WitnessError::MissingPrivateInput(name.clone()));
                }
            }
        } else if !circuit.private_inputs.is_empty() {
            return Err(WitnessError::NoPrivateInputsProvided);
        }
        Ok(())
    }

    fn execute_gate(&mut self, gate: &Gate) -> Result<(), WitnessError> {
        match gate {
            Gate::Const { output, value } => {
                self.wire_values.insert(output.clone(), *value);
                Ok(())
            }
            Gate::Add {
                output,
                left,
                right,
            } => {
                let left_val = self
                    .get_wire_value(left)
                    .ok_or_else(|| WitnessError::MissingWireValue(left.to_string()))?;
                let right_val = self
                    .get_wire_value(right)
                    .ok_or_else(|| WitnessError::MissingWireValue(right.to_string()))?;
                self.wire_values
                    .insert(output.clone(), left_val + right_val);
                Ok(())
            }
            Gate::Mul {
                output,
                left,
                right,
            } => {
                let left_val = self
                    .get_wire_value(left)
                    .ok_or_else(|| WitnessError::MissingWireValue(left.to_string()))?;
                let right_val = self
                    .get_wire_value(right)
                    .ok_or_else(|| WitnessError::MissingWireValue(right.to_string()))?;
                self.wire_values
                    .insert(output.clone(), left_val * right_val);
                Ok(())
            }
        }
    }
}
