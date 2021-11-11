use emulator::{
    cpu::{InstructionSet, CPU},
    parsers::{assembly::AssemblyInterpreter, Parser},
    ram::{VecBackedMemory},
};
use wasm_bindgen::prelude::*;

#[derive(Default)]
#[wasm_bindgen]
pub struct REPLBackend {
    cpu: CPU<VecBackedMemory>,
    interpreter: AssemblyInterpreter,
}

#[wasm_bindgen]
impl REPLBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpret_assembly(&mut self, assembly: String) -> String {
        match self.interpreter.parse(assembly.clone()) {
            Ok((instruction, size)) => {
                if let Err(e) = instruction.execute(&mut self.cpu, size) {
                    format!("Error: {:?}\n{}", e, self.cpu)
                } else {
                    format!("Ran assembly '{}'\n{}", assembly, self.cpu)
                }
            }
            Err(e) => format!("Parsing error: {:?}\n{}", e, self.cpu),
        }
    }
}

// Called when the wasm module is instantiated
#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    Ok(())
}
