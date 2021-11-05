//! Parsers convert an external source, such as 68k assembly or machine language, into instructions
//! that the emulator can understand.
//!
//! Any struct that implements the [`Parser`] trait can be used as a parser.

use std::num::{ParseIntError, TryFromIntError};

use crate::cpu::isa_68000::ISA68000;
pub mod assembly;
pub mod binary;

#[derive(Debug)]
pub enum ParseError {
    NoInstruction(String),
    UnknownInstruction(String),
    UnknownRegister(String),
    InvalidRegister {
        register: String,
        instruction: String,
        reason: String,
    },
    InvalidOperand {
        operand: String,
        instruction: String,
    },
    MissingOperand(String),
    UnknownOperandFormat {
        operand: String,
        instruction: String,
    },
    UnexpectedToken {
        token: char,
        instruction: String,
    },
    InvalidNumber {
        number: String,
        error: ParseIntError,
    },
    NumberTooLarge(TryFromIntError),
}

/// A parser
pub trait Parser<T> {
    fn parse(&mut self, source: T) -> Result<Vec<ISA68000>, ParseError>;
}

/// An incremental parser, suitable for a REPL
pub trait Interpreter<T> {
    fn parse_instruction(&mut self, source: T) -> Result<ISA68000, ParseError>;
}

impl<T, P> Parser<Vec<T>> for P
where
    P: Interpreter<T>,
    T: Sized,
{
    fn parse(&mut self, source: Vec<T>) -> Result<Vec<ISA68000>, ParseError> {
        let mut instructions = Vec::new();
        for item in source {
            instructions.push(self.parse_instruction(item)?);
        }
        Ok(instructions)
    }
}
