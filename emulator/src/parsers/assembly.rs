//! Parses assembly code

use crate::cpu::isa_68000::InstructionFor68000;

use super::{Interpreter, ParseError};

pub struct AssemblyInterpreter {}

impl Interpreter<String> for AssemblyInterpreter {
    fn parse_instruction(&mut self, source: String) -> Result<InstructionFor68000, ParseError> {
        unimplemented!();
    }
}
