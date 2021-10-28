//! Motorola 68k CPU emulation library.
pub mod cpu;
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
/// # use emulator::{cpu::{CPU, addressing::*}, ram::VecBackedMemory, OperandSize};
/// let mut cpu = CPU::<VecBackedMemory>::new(1024);
/// let address = AddressMode::Absolute { address: 0xABC, size: OperandSize::Word };
/// address.get_value(&mut cpu);
/// ```
///
/// `M68kInteger` is a wrapper enum which represents a byte, word, or long (internally as a Rust u8, u16, or u32).
#[derive(Debug, Copy, Clone)] // remove if perf issue
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
