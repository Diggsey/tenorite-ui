use smallbitvec::{SmallBitVec, sbvec};
use serde_json;
use maplit::btreemap;
use serde_derive::{Serialize, Deserialize};

use crate::library::{Library, ComponentMetadata};
use crate::component::{Component, Schema, PropertyError, Shape, FieldSchema, FieldType};

pub const CATEGORY: &'static str = "Gates";

#[derive(Debug, Copy, Clone)]
enum NaryGateType {
    And,
    Or,
    Xor,
    Parity,
}

impl NaryGateType {
    fn image_name(self, inverted: bool) -> &'static str {
        match (self, inverted) {
            (NaryGateType::And, false) => "and_gate",
            (NaryGateType::Or, false) => "or_gate",
            (NaryGateType::Xor, false) => "xor_gate",
            (NaryGateType::Parity, false) => "odd_parity",
            (NaryGateType::And, true) => "nand_gate",
            (NaryGateType::Or, true) => "nor_gate",
            (NaryGateType::Xor, true) => "xnor_gate",
            (NaryGateType::Parity, true) => "even_parity",
        }
    }
}

#[derive(Debug, Clone)]
struct NaryGate {
    type_: NaryGateType,
    invert_output: bool,
    num_inputs: u32,
    num_bits: u32,
    invert_inputs: SmallBitVec,
}

impl NaryGate {
    fn new(type_: NaryGateType) -> Self {
        Self {
            type_,
            invert_output: false,
            num_inputs: 2,
            num_bits: 1,
            invert_inputs: sbvec![false; 2],
        }
    }
}

#[derive(Serialize, Deserialize)]
enum YesNo {
    Yes,
    No
}

impl From<YesNo> for bool {
    fn from(v: YesNo) -> bool {
        match v {
            YesNo::Yes => true,
            YesNo::No => false,
        }
    }
}

impl From<bool> for YesNo {
    fn from(v: bool) -> YesNo {
        match v {
            true => YesNo::Yes,
            false => YesNo::No,
        }
    }
}

impl Component for NaryGate {
    fn schema(&self) -> Schema {
        let mut result = btreemap!{
            "invert_output".into() => FieldSchema {
                read_only: false,
                type_: FieldType::for_enum(&[YesNo::No, YesNo::Yes]),
                name: "Invert output".into(),
                description: None,
            },
            "num_inputs".into() => FieldSchema {
                read_only: false,
                type_: FieldType::Integer { min: 2, max: 32 },
                name: "Number of inputs".into(),
                description: None,
            },
            "num_bits".into() => FieldSchema {
                read_only: false,
                type_: FieldType::Integer { min: 1, max: 256 },
                name: "Data bits".into(),
                description: None,
            },
        };

        for i in 0..self.num_inputs {
            let id = format!("invert_input_{}", i);
            result.insert(id.into(), FieldSchema {
                read_only: false,
                type_: FieldType::for_enum(&[YesNo::No, YesNo::Yes]),
                name: format!("Invert input {}", i).into(),
                description: None,
            });
        }

        result
    }
    fn set_property(&mut self, name: &str, value: serde_json::Value) -> Result<(), PropertyError> {
        match name {
            "invert_output" => {
                self.invert_output = serde_json::from_value::<YesNo>(value)
                    .map_err(|e| PropertyError::from_serde(e, name))?
                    .into();
                Ok(())
            },
            "num_inputs" => {
                self.num_inputs = serde_json::from_value(value)
                    .map_err(|e| PropertyError::from_serde(e, name))?;
                self.invert_inputs.resize(self.num_inputs as usize, false);
                Ok(())
            },
            "num_bits" => {
                self.num_inputs = serde_json::from_value(value)
                    .map_err(|e| PropertyError::from_serde(e, name))?;
                Ok(())
            },
            _ if name.starts_with("invert_input_") => {
                let v = serde_json::from_value::<YesNo>(value)
                    .map_err(|e| PropertyError::from_serde(e, name))?
                    .into();
                for i in 0..self.num_inputs {
                    let id = format!("invert_input_{}", i);
                    if id == name {
                        self.invert_inputs.set(i as usize, v);
                        return Ok(());
                    }
                }
                Err(PropertyError::unknown(name))
            },
            _ => Err(PropertyError::unknown(name))
        }
    }
    fn get_property(&self, name: &str) -> Option<serde_json::Value> {
        match name {
            "invert_output" => {
                serde_json::to_value::<YesNo>(self.invert_output.into()).ok()
            },
            "num_inputs" => {
                serde_json::to_value(self.num_inputs).ok()
            },
            "num_bits" => {
                serde_json::to_value(self.num_bits).ok()
            },
            _ if name.starts_with("invert_input_") => {
                for i in 0..self.num_inputs {
                    let id = format!("invert_input_{}", i);
                    if id == name {
                        let v = self.invert_inputs[i as usize];
                        return serde_json::to_value::<YesNo>(v.into()).ok();
                    }
                }
                None
            },
            _ => None
        }
    }
    fn get_shape(&self) -> Shape {
        Shape {
            width: 3,
            height: 3,
            pins: vec![],
            image_name: self.type_.image_name(self.invert_output).into(),
        }
    }
}

pub fn library() -> Library {
    let mut result = Library::new();
    result.add(
        ComponentMetadata::new("or_gate", "OR Gate", CATEGORY, "Logical OR gate"),
        || Box::new(NaryGate::new(NaryGateType::Or))
    );
    result
}