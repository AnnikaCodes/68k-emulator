// CPU emulation
use crate::ram::Memory;
pub mod addressing;
pub mod registers;
use registers::*;

use self::addressing::AddressMode;

#[derive(Debug)]
pub enum CPUError {
    MemoryOutOfBoundsAccess(u32),
}

/// byte, word, or long
pub trait SizedValue {}

/// byte
impl SizedValue for u8 {}
/// word
impl SizedValue for u16 {}
/// long
impl SizedValue for u32 {}

pub trait Addressable<T: SizedValue> {
    /// Returns the value of the address.
    fn get_value(&self, cpu: &mut CPU<impl Memory>) -> T;
}

/// A CPU instruction
pub trait Instruction {
    fn execute(&self, cpu: &mut CPU<impl Memory>) -> Result<(), CPUError>;
}

pub struct CPU<M: Memory> {
    pub registers: Registers,
    pub memory: M,
}

impl<M> CPU<M>
where
    M: Memory,
{
    pub fn new(ram_size_in_bytes: usize) -> Self {
        Self {
            registers: Registers::new(),
            memory: M::new(ram_size_in_bytes),
        }
    }

    /// Syntactic sugar
    pub fn run_instruction(&mut self, instruction: impl Instruction) -> Result<(), CPUError> {
        instruction.execute(self)
    }

    pub fn get_address_value(&self, addr: AddressMode) -> Result<u32, CPUError> {
        addr.get_value(self)
    }
}
