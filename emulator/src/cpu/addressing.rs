//! Emulates the 68000's addressing modes.
//!
//! Refer to http://www.scarpaz.com/Attic/Didattica/Scarpazza-2005-68k-1-addressing.pdf
//! and http://faculty.cs.niu.edu/~winans/CS463/notes/amodes.pdf for reference on how these work.

use super::registers::*;
use super::{CPUError, CPU};

#[derive(Debug)] // remove if perf issue
pub enum OperandSize {
    Byte,
    Word,
    Long,
}

#[derive(Debug, Copy, Clone)]
pub enum IndexScale {
    One = 1,
    Two = 2,
    Four = 4,
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
    RegisterIndirectWithDisplacement {
        register: AddressRegister,
        displacement: u16,
    },
    RegisterIndirectIndexed {
        displacement: u32,
        address_register: AddressRegister,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },

    // Memory-based addressing
    MemoryPostIndexed {
        base_displacement: u32,
        outer_displacement: u32,
        address_register: AddressRegister,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },
    MemoryPreIndexed {
        base_displacement: u32,
        outer_displacement: u32,
        address_register: AddressRegister,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },

    // Program counter-based addressing
    ProgramCounterIndirectWithDisplacement {
        displacement: u16,
    },
    ProgramCounterIndirectWithIndex {
        displacement: u16,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },
    ProgramCounterMemoryIndirectPreIndexed {
        base_displacement: u16,
        outer_displacement: u32,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },
    ProgramCounterMemoryIndirectPostIndexed {
        base_displacement: u16,
        outer_displacement: u32,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },

    // Absolute addressing
    Absolute {
        address: u32,
    },

    // Immediate addressing
    Immediate {
        value: u32,
    },
}

impl AddressMode {
    pub fn get_value(&self, cpu: &CPU<impl crate::ram::Memory>) -> Result<u32, CPUError> {
        match self {
            AddressMode::Absolute { address } => cpu.memory.read_long(*address),
            AddressMode::Immediate { value } => Ok(*value),
            AddressMode::RegisterDirect { register } => Ok(cpu.registers.get(*register)),
            AddressMode::RegisterIndirect { register } => cpu
                .memory
                .read_long(cpu.registers.get_address_register(*register)),
            AddressMode::RegisterIndirectIndexed {
                displacement,
                address_register,
                index_register,
                index_scale,
                ..
            } => {
                let base_address = cpu.registers.get_address_register(*address_register);
                let index_value = cpu.registers.get(*index_register) * (*index_scale as u32);
                let operand_address = base_address + displacement + index_value;
                cpu.memory.read_long(operand_address)
            }
            _ => unimplemented!("Addressing mode {:?}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::{Memory, VecBackedMemory};

    // This saves us from having to hardcode lots of values or declare the same variables in each test function.
    // It also means that we don't have to worry about typing the wrong value.
    static VALUE: u32 = 0xFACEBEEF;
    static ADDRESS: u32 = 0x42;
    static DISPLACEMENT: u16 = 0xA3;
    static OUTER_DISPLACEMENT: u32 = 0x1A;
    static INDEX: u32 = 4;
    static ADDRESS_REGISTER: AddressRegister = AddressRegister::A0;
    static DATA_REGISTER: DataRegister = DataRegister::D0;

    #[test]
    fn register_direct() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let address = AddressMode::RegisterDirect {
            register: Register::Data(DATA_REGISTER),
        };

        cpu.registers.set_data_register(DATA_REGISTER, VALUE);
        assert_eq!(address.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn register_indirect() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::RegisterIndirect {
            register: ADDRESS_REGISTER,
        };

        cpu.memory.write_long(ADDRESS, VALUE).unwrap();
        cpu.registers
            .set_address_register(ADDRESS_REGISTER, ADDRESS);
        assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn register_indirect_with_postincrement() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        for size in [OperandSize::Byte, OperandSize::Word, OperandSize::Long] {
            let byte_offset = size.size_in_bytes();
            let mode = AddressMode::RegisterIndirectPostIncrement {
                register: ADDRESS_REGISTER,
                size,
            };

            // will this actuall work for _ALL_ operand sizes?
            // or do we need to write_byte, write_word, write_long, etc?
            cpu.memory.write_long(ADDRESS, VALUE).unwrap();
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);

            assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS + byte_offset
            );
        }
    }

    #[test]
    fn register_indirect_with_predecrement() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        for size in [OperandSize::Byte, OperandSize::Word, OperandSize::Long] {
            let byte_offset = size.size_in_bytes();
            let mode = AddressMode::RegisterIndirectPreDecrement {
                register: ADDRESS_REGISTER,
                size,
            };

            cpu.memory.write_long(ADDRESS - byte_offset, VALUE).unwrap();
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);

            assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS - byte_offset
            );
        }
    }

    #[test]
    /// "if the address register is stack pointer and operand size is byte,
    /// the address is incremented by 2 to preserve alignment" (Scarpazza)
    fn register_indirect_stack_pointer_special_case() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);

        let post_incr = AddressMode::RegisterIndirectPostIncrement {
            register: AddressRegister::SP,
            size: OperandSize::Byte,
        };
        cpu.registers
            .set_address_register(AddressRegister::SP, ADDRESS);
        post_incr.get_value(&cpu).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::SP),
            ADDRESS + 2
        ); // not +1

        let pre_decr = AddressMode::RegisterIndirectPreDecrement {
            register: AddressRegister::SP,
            size: OperandSize::Byte,
        };
        cpu.registers
            .set_address_register(AddressRegister::SP, ADDRESS);
        pre_decr.get_value(&cpu).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::SP),
            ADDRESS - 2
        ); // not -1
    }

    #[test]
    fn register_indirect_with_displacement() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::RegisterIndirectWithDisplacement {
            displacement: DISPLACEMENT,
            register: ADDRESS_REGISTER,
        };

        cpu.registers.set_address_register(ADDRESS_REGISTER, VALUE);
        cpu.memory
            .write_long(ADDRESS + DISPLACEMENT as u32, VALUE)
            .unwrap();
        assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn register_indirect_indexed() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::RegisterIndirectIndexed {
            address_register: ADDRESS_REGISTER,
            index_register: Register::Data(DATA_REGISTER),
            size: OperandSize::Word,
            index_scale: IndexScale::Two,
            displacement: DISPLACEMENT as u32,
        };

        cpu.registers
            .set_address_register(ADDRESS_REGISTER, ADDRESS);
        cpu.registers.set_data_register(DATA_REGISTER, INDEX);
        cpu.memory
            .write_long(ADDRESS + DISPLACEMENT as u32 + (INDEX * 2), VALUE)
            .unwrap();

        assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn memory_post_indexed() {
        let initial_address = 0xAAAAAA;
        for size in [OperandSize::Byte, OperandSize::Word, OperandSize::Long] {
            let mut cpu = CPU::<VecBackedMemory>::new(1024);
            let mode = AddressMode::MemoryPostIndexed {
                size,
                base_displacement: DISPLACEMENT as u32,
                outer_displacement: OUTER_DISPLACEMENT,
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32;
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory
                .write_long(ADDRESS + (INDEX * 4) + OUTER_DISPLACEMENT, VALUE)
                .unwrap();

            assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
        }
    }

    #[test]
    fn memory_pre_indexed() {
        let initial_address = 0xAAAAAA;
        for size in [OperandSize::Byte, OperandSize::Word, OperandSize::Long] {
            let mut cpu = CPU::<VecBackedMemory>::new(1024);
            let mode = AddressMode::MemoryPreIndexed {
                size,
                base_displacement: DISPLACEMENT as u32,
                outer_displacement: OUTER_DISPLACEMENT,
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32 + (INDEX * 4);
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory
                .write_long(ADDRESS + OUTER_DISPLACEMENT, VALUE)
                .unwrap();

            assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
        }
    }

    #[test]
    fn program_counter_indirect_with_displacement() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::ProgramCounterIndirectWithDisplacement {
            displacement: DISPLACEMENT,
        };

        cpu.registers.set(Register::ProgramCounter, VALUE);
        cpu.memory
            .write_long(ADDRESS + DISPLACEMENT as u32, VALUE)
            .unwrap();
        assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn program_counter_indirect_indexed() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::ProgramCounterIndirectWithIndex {
            displacement: DISPLACEMENT,
            index_register: Register::Data(DATA_REGISTER),
            index_scale: IndexScale::Two,
            size: OperandSize::Word,
        };

        cpu.registers.set(Register::ProgramCounter, VALUE);
        cpu.memory
            .write_long(ADDRESS + DISPLACEMENT as u32 + (INDEX * 2), VALUE)
            .unwrap();
        assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn program_counter_memory_post_indexed() {
        let initial_address = 0xAAAAAA;
        for size in [OperandSize::Byte, OperandSize::Word, OperandSize::Long] {
            let mut cpu = CPU::<VecBackedMemory>::new(1024);
            let mode = AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                size,
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
            };

            cpu.registers.set(Register::ProgramCounter, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32;
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory
                .write_long(ADDRESS + (INDEX * 4) + OUTER_DISPLACEMENT, VALUE)
                .unwrap();

            assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
        }
    }

    #[test]
    fn program_counter_memory_pre_indexed() {
        let initial_address = 0xAAAAAA;
        for size in [OperandSize::Byte, OperandSize::Word, OperandSize::Long] {
            let mut cpu = CPU::<VecBackedMemory>::new(1024);
            let mode = AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                size,
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
            };

            cpu.registers.set(Register::ProgramCounter, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32 + (INDEX * 4);
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory
                .write_long(ADDRESS + OUTER_DISPLACEMENT, VALUE)
                .unwrap();

            assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
        }
    }

    #[test]
    fn absolute() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);

        // TODO: do these need to return smaller than u32?
        let mode = AddressMode::Absolute { address: ADDRESS };

        cpu.memory.write_long(ADDRESS, VALUE).unwrap();
        assert_eq!(mode.get_value(&cpu).unwrap(), VALUE);
    }

    #[test]
    fn immediate() {
        let cpu = CPU::<VecBackedMemory>::new(1024);
        let address = AddressMode::Immediate { value: VALUE };

        assert_eq!(address.get_value(&cpu).unwrap(), VALUE);
    }
}
