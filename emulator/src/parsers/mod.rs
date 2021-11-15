//! Parsers convert an external source, such as 68k assembly or machine language, into instructions
//! that the emulator can understand.
//!
//! Any struct that implements the [`Parser`] trait can be used as a parser.

use std::num::{ParseIntError, TryFromIntError};

use crate::{cpu::isa_68000::Instruction, OperandSize};
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
    OperandSizeMismatch {
        instruction: String,
        source_size: OperandSize,
        dest_size: OperandSize,
    },
    NumberTooLarge(TryFromIntError),
    OpcodeParsingError(m68kdecode::DecodingError),
    InvalidOperandSize(i32),
}

impl From<m68kdecode::DecodingError> for ParseError {
    fn from(error: m68kdecode::DecodingError) -> Self {
        Self::OpcodeParsingError(error)
    }
}

/// A parser
pub trait Parser<T> {
    /// u8 is the number of bytes used for the opcode
    fn parse(&mut self, source: T) -> Result<(Instruction, OperandSize, u32), ParseError>;
}
