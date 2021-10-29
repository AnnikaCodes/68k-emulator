//! Parses binary machine code

use crate::cpu::isa_68000::InstructionFor68000;

use super::{Parser, ParseError};

pub struct MachineCodeParser {}

impl Parser<[u8]> for MachineCodeParser {
    fn parse(&mut self, source: [u8]) -> Result<InstructionFor68000, ParseError> {
        unimplemented!();
    }
}
