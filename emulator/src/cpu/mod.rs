// CPU emulation
use crate::{ram::Memory, M68kInteger};
pub mod addressing;
pub mod instructions;
pub mod registers;
use registers::*;

use self::{addressing::AddressMode, instructions::Instruction};

#[derive(Debug)]
pub enum CPUError {
    MemoryOutOfBoundsAccess(u32),
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

    pub fn get_address_value(&mut self, addr: AddressMode) -> Result<M68kInteger, CPUError> {
        addr.get_value(self)
    }
}
