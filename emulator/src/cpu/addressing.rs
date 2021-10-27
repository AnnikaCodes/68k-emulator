//! Emulates the 68000's addressing modes.
//!
//! Refer to http://www.scarpaz.com/Attic/Didattica/Scarpazza-2005-68k-1-addressing.pdf
//! for reference on how these work.

use super::registers::*;
use super::CPU;

#[derive(Debug)] // remove if perf issue
pub enum OperandSize {
    Byte,
    Word,
    Long,
}

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

impl AddressMode {
    pub fn get_value(&self, cpu: &CPU<impl crate::ram::Memory>) -> u32 {
        match self {
            AddressMode::Immediate { value } => *value,
            AddressMode::RegisterDirect { register } => cpu.registers.get(register),
            _ => unimplemented!("Addressing mode {:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::VecBackedMemory;

    #[test]
    fn register_direct() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let address = AddressMode::RegisterDirect {
            register: Register::Data(DataRegister::D0),
        };

        cpu.registers
            .set_data_register(DataRegister::D0, 0xDEADBEEF);
        assert_eq!(address.get_value(&cpu), 0xDEADBEEF);
    }

    #[test]
    fn immediate() {
        let cpu = CPU::<VecBackedMemory>::new(1024);
        let address = AddressMode::Immediate { value: 0xDEADBEEF };

        assert_eq!(address.get_value(&cpu), 0xDEADBEEF);
    }

    // TODO: implement RAM and test address modes that require RAM
}
