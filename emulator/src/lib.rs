#![feature(slice_pattern)]
//! Motorola 68k CPU emulation library.

use std::ops::{Add, Sub};

use cpu::CPUError;
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
/// # use emulator::{cpu::{CPU, CPUError, addressing::*}, ram::{Memory, VecBackedMemory}, OperandSize, M68kInteger};
/// # fn test() -> Result<(), CPUError> {
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

    pub fn from_size_in_bytes(size: i32) -> Result<Self, CPUError> {
        match size {
            1 => Ok(OperandSize::Byte),
            2 => Ok(OperandSize::Word),
            4 => Ok(OperandSize::Long),
            _ => Err(CPUError::InvalidOperandSize(size)),
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

    pub fn check_size(&self, size: OperandSize) -> Result<(), CPUError> {
        if !self.is_size(size) {
            Err(CPUError::WrongSizeInteger(*self))
        } else {
            Ok(())
        }
    }

    pub fn wrapping_mul(self, other: M68kInteger) -> M68kInteger {
        match (self, other) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => M68kInteger::Byte(a.wrapping_mul(b)),
            (M68kInteger::Word(a), M68kInteger::Word(b)) => M68kInteger::Word(a.wrapping_mul(b)),
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a.wrapping_mul(b)),
            // TODO: this should probably not panic
            _ => panic!(
                "M68kInteger::wrapping_mul: invalid operands {:?} and {:?}",
                self, other
            ),
        }
    }

    pub fn rotate_left(self, amount: M68kInteger) -> M68kInteger {
        match (self, amount) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => {
                M68kInteger::Byte(a.rotate_left(b.into()))
            }
            (M68kInteger::Word(a), M68kInteger::Word(b)) => {
                M68kInteger::Word(a.rotate_left(b.into()))
            }
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a.rotate_left(b)),
            // TODO: this should probably not panic
            _ => panic!(
                "M68kInteger::rotate_left: invalid operands {:?} and {:?}",
                self, amount
            ),
        }
    }

    // TODO: make these into macros?
    pub fn or(self, other: M68kInteger) -> M68kInteger {
        match (self, other) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => M68kInteger::Byte(a | b),
            (M68kInteger::Word(a), M68kInteger::Word(b)) => M68kInteger::Word(a | b),
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a | b),
            _ => panic!(
                "M68kInteger::or: invalid operands {:?} and {:?}",
                self, other
            ),
        }
    }

    pub fn and(self, other: M68kInteger) -> M68kInteger {
        match (self, other) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => M68kInteger::Byte(a & b),
            (M68kInteger::Word(a), M68kInteger::Word(b)) => M68kInteger::Word(a & b),
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a & b),
            _ => panic!(
                "M68kInteger::and: invalid operands {:?} and {:?}",
                self, other
            ),
        }
    }

    pub fn xor(self, other: M68kInteger) -> M68kInteger {
        match (self, other) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => M68kInteger::Byte(a ^ b),
            (M68kInteger::Word(a), M68kInteger::Word(b)) => M68kInteger::Word(a ^ b),
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a ^ b),
            _ => panic!(
                "M68kInteger::and: invalid operands {:?} and {:?}",
                self, other
            ),
        }
    }
}

impl Add for M68kInteger {
    type Output = M68kInteger;

    fn add(self, other: M68kInteger) -> M68kInteger {
        match (self, other) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => M68kInteger::Byte(a.wrapping_add(b)),
            (M68kInteger::Word(a), M68kInteger::Word(b)) => M68kInteger::Word(a.wrapping_add(b)),
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a.wrapping_add(b)),
            _ => panic!(
                "M68kInteger::add: invalid operands {:?} and {:?}",
                self, other
            ),
        }
    }
}

impl Sub for M68kInteger {
    type Output = M68kInteger;

    fn sub(self, other: M68kInteger) -> M68kInteger {
        match (self, other) {
            (M68kInteger::Byte(a), M68kInteger::Byte(b)) => M68kInteger::Byte(a.wrapping_sub(b)),
            (M68kInteger::Word(a), M68kInteger::Word(b)) => M68kInteger::Word(a.wrapping_sub(b)),
            (M68kInteger::Long(a), M68kInteger::Long(b)) => M68kInteger::Long(a.wrapping_sub(b)),
            _ => panic!(
                "M68kInteger::sub: invalid operands {:?} and {:?}",
                self, other
            ),
        }
    }
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
