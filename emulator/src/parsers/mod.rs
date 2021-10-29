//! Parsers convert an external source, such as 68k assembly or machine language, into instructions
//! that the emulator can understand.
//!
//! Any struct that implements the [`Parser`] trait can be used as a parser.

use crate::cpu::isa_68000::InstructionFor68000;
pub mod assembly;

#[derive(Debug)]
pub enum ParseError {
    NoInstruction(String),
    UnknownInstruction(String),
    MissingOperand(String),
}

/// A parser
pub trait Parser<T> {
    fn parse(&mut self, source: T) -> Result<Vec<InstructionFor68000>, ParseError>;
}

/// An incremental parser, suitable for a REPL
pub trait Interpreter<T> {
    fn parse_instruction(&mut self, source: T) -> Result<InstructionFor68000, ParseError>;
}

impl<T, P> Parser<Vec<T>> for P
where
    P: Interpreter<T>,
    T: Sized,
{
    fn parse(&mut self, source: Vec<T>) -> Result<Vec<InstructionFor68000>, ParseError> {
        let mut instructions = Vec::new();
        for item in source {
            instructions.push(self.parse_instruction(item)?);
        }
        Ok(instructions)
    }
}
