// CPU emulation
use crate::ram::{Memory, VecBackedMemory};

type RegisterValue = u32;

#[derive(Debug)]
pub enum CPUError {
    MemoryOutOfBoundsAccess(u32),
}

#[derive(Debug)] // remove if perf issue
pub enum OperandSize {
    Byte,
    Word,
    Long,
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

#[derive(Debug)] // remove if perf issue
pub enum Register {
    Data(DataRegister),
    Address(AddressRegister),
}

#[derive(Debug)] // remove if perf issue
pub enum DataRegister {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
}

#[derive(Debug)] // remove if perf issue
pub enum AddressRegister {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
}

/// Implements the 68000's many address modes.
/// See http://www.scarpaz.com/Attic/Didattica/Scarpazza-2005-68k-1-addressing.pdf
#[derive(Debug)] // remove if perf issue
pub enum AddressMode {
    // Register-based addressing
    RegisterDirect {
        register: Register,
    },
    RegisterIndirect {
        register: AddressRegister,
    },
    RegisterIndirectPostIncrement {
        register: AddressRegister,
        size: OperandSize,
    },
    RegisterIndirectPreDecrement {
        register: AddressRegister,
        size: OperandSize,
    },
    RegisterIndirectIndex {
        displacement: u32,
        address_register: AddressRegister,
        // TODO: Is this any register or can it only be an Address Register?
        index_register: Register,
        size: OperandSize,
    },

    // Memory-based addressing
    MemoryPostIndexed {
        base_displacement: u32,
        outer_displacement: u32,
        address_register: AddressRegister,
        index_register: Register,
        size: OperandSize,
    },
    MemoryPreIndexed {
        base_displacement: u32,
        outer_displacement: u32,
        address_register: AddressRegister,
        index_register: Register,
        size: OperandSize,
    },

    // Program counter-based addressing
    // this guy can't write...
    ProgramCounterIndirectWithDisplacement {
        displacement: u32,
    },
    ProgramCounterIndirectWithIndex {
        displacement: u32,
        index_register: Register,
        size: OperandSize,
    },
    ProgramCounterMemoryIndirect {
        base_displacement: u32,
        outer_displacement: u32,
        index_register: Register,
        size: OperandSize,
    },

    // Absolute addressing
    AbsoluteShort {
        address: u16,
    },
    AbsoluteLong {
        address: u32,
    },

    // Immediate addressing
    Immediate {
        value: u32,
    },
}

/// A CPU instruction
pub trait Instruction {
    fn execute(&self, cpu: &mut CPU<impl Memory>) -> Result<(), CPUError>;
}

pub struct Registers {
    // Data registers
    d0: RegisterValue,
    d1: RegisterValue,
    d2: RegisterValue,
    d3: RegisterValue,
    d4: RegisterValue,
    d5: RegisterValue,
    d6: RegisterValue,
    d7: RegisterValue,

    // Address registers
    a0: RegisterValue,
    a1: RegisterValue,
    a2: RegisterValue,
    a3: RegisterValue,
    a4: RegisterValue,
    a5: RegisterValue,
    a6: RegisterValue,
    a7: RegisterValue,

    /// Program counter
    pc: RegisterValue,

    /// Status register (top 8 bytes = system byte, bottom 8 bytes = CCR)
    status: u16,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers {
            d0: 0,
            d1: 0,
            d2: 0,
            d3: 0,
            d4: 0,
            d5: 0,
            d6: 0,
            d7: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            a6: 0,
            a7: 0,
            pc: 0,
            status: 0,
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, register: Register) -> RegisterValue {
        match register {
            Register::Data(DataRegister::D0) => self.d0,
            Register::Data(DataRegister::D1) => self.d1,
            Register::Data(DataRegister::D2) => self.d2,
            Register::Data(DataRegister::D3) => self.d3,
            Register::Data(DataRegister::D4) => self.d4,
            Register::Data(DataRegister::D5) => self.d5,
            Register::Data(DataRegister::D6) => self.d6,
            Register::Data(DataRegister::D7) => self.d7,
            Register::Address(AddressRegister::A0) => self.a0,
            Register::Address(AddressRegister::A1) => self.a1,
            Register::Address(AddressRegister::A2) => self.a2,
            Register::Address(AddressRegister::A3) => self.a3,
            Register::Address(AddressRegister::A4) => self.a4,
            Register::Address(AddressRegister::A5) => self.a5,
            Register::Address(AddressRegister::A6) => self.a6,
            Register::Address(AddressRegister::A7) => self.a7,
        }
    }

    pub fn set(&mut self, register: Register, new_value: RegisterValue) {
        match register {
            Register::Data(DataRegister::D0) => self.d0 = new_value,
            Register::Data(DataRegister::D1) => self.d1 = new_value,
            Register::Data(DataRegister::D2) => self.d2 = new_value,
            Register::Data(DataRegister::D3) => self.d3 = new_value,
            Register::Data(DataRegister::D4) => self.d4 = new_value,
            Register::Data(DataRegister::D5) => self.d5 = new_value,
            Register::Data(DataRegister::D6) => self.d6 = new_value,
            Register::Data(DataRegister::D7) => self.d7 = new_value,
            Register::Address(AddressRegister::A0) => self.a0 = new_value,
            Register::Address(AddressRegister::A1) => self.a1 = new_value,
            Register::Address(AddressRegister::A2) => self.a2 = new_value,
            Register::Address(AddressRegister::A3) => self.a3 = new_value,
            Register::Address(AddressRegister::A4) => self.a4 = new_value,
            Register::Address(AddressRegister::A5) => self.a5 = new_value,
            Register::Address(AddressRegister::A6) => self.a6 = new_value,
            Register::Address(AddressRegister::A7) => self.a7 = new_value,
        }
    }
}

pub struct CPU<M: Memory> {
    pub registers: Registers,
    pub memory: M,
}

impl<RAM> CPU<RAM>
where
RAM: Memory,
{
    pub fn new(ram_size_in_bytes: usize) -> Self {
        Self {
            registers: Registers::new(),
            memory: RAM::new(ram_size_in_bytes),
        }
    }

    pub fn set_pc(&mut self, pc: RegisterValue) {
        self.registers.pc = pc;
    }

    /// Syntactic sugar
    pub fn run_instruction(&mut self, instruction: impl Instruction) -> Result<(), CPUError> {
        instruction.execute(self)
    }

    pub fn get_address_value(&self, addr: AddressMode) -> u32 {
        match addr {
            AddressMode::Immediate { value } => value,
            AddressMode::RegisterDirect { register } => self.registers.get(register),
            _ => unimplemented!("Address mode {:?}", addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_pc() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);

        assert_eq!(cpu.registers.pc, 0x0);
        cpu.set_pc(0xDEADBEEF);
        assert_eq!(cpu.registers.pc, 0xDEADBEEF);
    }
}

#[cfg(test)]
mod address_modes {
    use super::*;

    #[test]
    fn register_direct() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::RegisterDirect {
            register: Register::Data(DataRegister::D0),
        };
        cpu.registers
            .set(Register::Data(DataRegister::D0), 0xDEADBEEF);

        assert_eq!(cpu.get_address_value(mode), 0xDEADBEEF);
    }

    #[test]
    fn immediate() {
        let cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::Immediate { value: 0xDEADBEEF };

        assert_eq!(cpu.get_address_value(mode), 0xDEADBEEF);
    }

    // TODO: implement RAM and test address modes that require RAM
}
