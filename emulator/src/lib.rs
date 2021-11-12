#![feature(slice_pattern)]
//! Motorola 68k CPU emulation library.

use parsers::ParseError;

#[derive(Debug)]
pub enum EmulationError {
    MemoryOutOfBoundsAccess(u32),
    WriteToReadOnly(String),
    WrongSizeInteger(M68kInteger),
    InvalidOperandSize(i32),
    Parsing(ParseError),
}
impl From<ParseError> for EmulationError {
    fn from(err: ParseError) -> Self {
        EmulationError::Parsing(err)
    }
}

pub mod cpu;
pub mod parsers;
pub mod ram;

/// The 68000 handles three sizes of data:
/// - 8-bit bytes
/// - 16-bit words
/// - 32-bit longs
///
/// All (or most? I haven't yet found clear and concise documentation) operations work on all these sizes.
///
/// I've created several types to deal with this.
///
/// `OperandSize` is an enum which represents the size of the value
/// in an addressing mode (as specified in an assembler or machine code instruction).
/// For example, to read 16 bits from memory at the address 0xABC, use the following addressing:
/// ```
/// # use emulator::{EmulationError, cpu::{CPU, addressing::*}, ram::{Memory, VecBackedMemory}, OperandSize, M68kInteger};
/// # fn test() -> Result<(), EmulationError> {
///     let mut cpu = CPU::<VecBackedMemory>::new(1024);
///     cpu.memory.write_word(0xABC, 0xBEEF);
///
///     let address = AddressMode::Absolute { address: 0xABC };
///     assert_eq!(address.get_value(&mut cpu, OperandSize::Word)?, M68kInteger::Word(0xBEEF));
/// #     Ok(())
/// # }
/// ```
///
/// `M68kInteger` is a wrapper enum which represents a byte, word, or long (internally as a Rust u8, u16, or u32).
#[derive(Debug, Copy, Clone, PartialEq, Eq)] // remove if perf issue
pub enum OperandSize {
    Byte,
    Word,
    Long,
}

impl OperandSize {
    pub fn size_in_bytes(&self) -> u32 {
        match self {
            OperandSize::Byte => 1,
            OperandSize::Word => 2,
            OperandSize::Long => 4,
        }
    }

    pub fn from_size_in_bytes(size: i32) -> Result<Self, EmulationError> {
        match size {
            1 => Ok(OperandSize::Byte),
            2 => Ok(OperandSize::Word),
            4 => Ok(OperandSize::Long),
            _ => Err(EmulationError::InvalidOperandSize(size)),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)] // remove if perf issue
pub enum M68kInteger {
    Byte(u8),
    Word(u16),
    Long(u32),
}

impl From<M68kInteger> for u32 {
    fn from(val: M68kInteger) -> u32 {
        match val {
            M68kInteger::Byte(b) => b as u32,
            M68kInteger::Word(w) => w as u32,
            M68kInteger::Long(l) => l,
        }
    }
}

/// Implements an operation on all three sizes of data.
///
/// Requires that both operands be of the same size.
macro_rules! operation_impl {
    ($name:ident, |$a:ident, $b:ident| $op:expr) => {
        pub fn $name(&self, other: M68kInteger) -> M68kInteger {
            match (self, other) {
                (&M68kInteger::Byte($a), M68kInteger::Byte($b)) => M68kInteger::Byte($op),
                (&M68kInteger::Word($a), M68kInteger::Word($b)) => M68kInteger::Word($op),
                (&M68kInteger::Long($a), M68kInteger::Long($b)) => M68kInteger::Long($op),
                _ => panic!("Mismatched operand sizes"),
            }
        }
    };
}

impl M68kInteger {
    pub fn size(&self) -> OperandSize {
        match self {
            M68kInteger::Byte(_) => OperandSize::Byte,
            M68kInteger::Word(_) => OperandSize::Word,
            M68kInteger::Long(_) => OperandSize::Long,
        }
    }

    pub fn is_size(&self, size: OperandSize) -> bool {
        size == self.size()
    }

    pub fn check_size(&self, size: OperandSize) -> Result<(), EmulationError> {
        if !self.is_size(size) {
            Err(EmulationError::WrongSizeInteger(*self))
        } else {
            Ok(())
        }
    }

    operation_impl!(wrapping_add, |a, b| a.wrapping_add(b));
    operation_impl!(wrapping_sub, |a, b| a.wrapping_sub(b));
    operation_impl!(wrapping_mul, |a, b| a.wrapping_mul(b));
    operation_impl!(rotate_left, |a, b| a.rotate_left(b.into()));
    operation_impl!(and, |a, b| a & b);
    operation_impl!(or, |a, b| a | b);
    operation_impl!(xor, |a, b| a ^ b);
}

pub fn hex_format_byte(byte: u8) -> String {
    format!("{:02X}", byte)
}

pub fn hex_format_word(word: u16) -> String {
    format!("{:04X}", word)
}

pub fn hex_format_long(long: u32) -> String {
    format!("{:08X}", long)
}
