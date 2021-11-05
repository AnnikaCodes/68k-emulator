use emulator::{
    cpu::CPU,
    parsers::{assembly::AssemblyInterpreter, Interpreter},
    ram::{Memory, VecBackedMemory},
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
        match self.interpreter.parse_instruction(assembly.clone()) {
            Ok(instruction) => {
                if let Err(e) = self.cpu.run_instruction(instruction) {
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
