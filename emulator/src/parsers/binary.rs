//! Parses binary machine code

use crate::{cpu::isa_68000::ISA68000, OperandSize};

use super::{ParseError, Parser};

#[derive(Default)]
pub struct MachineCodeParser;

impl Parser<Vec<u8>> for MachineCodeParser {
    fn parse(&mut self, _source: Vec<u8>) -> Result<Vec<(ISA68000, OperandSize)>, ParseError> {
        unimplemented!();
    }
}
