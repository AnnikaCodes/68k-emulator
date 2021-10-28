//! Emulates the 68000's addressing modes.
//!
//! Refer to http://www.scarpaz.com/Attic/Didattica/Scarpazza-2005-68k-1-addressing.pdf
//! and http://faculty.cs.niu.edu/~winans/CS463/notes/amodes.pdf for reference on how these work.

use crate::{M68kInteger, OperandSize};

use super::registers::*;
use super::{CPUError, CPU};

/// Index register scaling - the ONLY legal values for this are 1, 2, and 4.
#[derive(Debug, Copy, Clone)]
pub enum IndexScale {
    One = 1,
    Two = 2,
    Four = 4,
}

#[derive(Debug)] // remove if perf issue
pub enum AddressMode {
    // Register-based addressing
    RegisterDirect {
        register: Register,
        size: OperandSize,
    },
    RegisterIndirect {
        register: AddressRegister,
        size: OperandSize,
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
        size: OperandSize,
    },
    RegisterIndirectIndexed {
        displacement: u16,
        address_register: AddressRegister,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },

    // Memory-based addressing
    MemoryPostIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        address_register: AddressRegister,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },
    MemoryPreIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        address_register: AddressRegister,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },

    // Program counter-based addressing
    ProgramCounterIndirectWithDisplacement {
        displacement: u16,
        size: OperandSize,
    },
    ProgramCounterIndirectWithIndex {
        displacement: u16,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },
    ProgramCounterMemoryIndirectPreIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },
    ProgramCounterMemoryIndirectPostIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        index_register: Register,
        index_scale: IndexScale,
        size: OperandSize,
    },

    // Absolute addressing
    Absolute {
        address: u32,
        size: OperandSize,
    },

    // Immediate addressing
    Immediate {
        value: u32,
        size: OperandSize,
    },
}

/// Gets the increment for a given register + value size
fn get_increment(register: AddressRegister, size: OperandSize) -> u32 {
    let minimum = match register {
        AddressRegister::A7 => 2,
        _ => 1,
    };

    let increment = size.size_in_bytes();
    if increment < minimum {
        minimum
    } else {
        increment
    }
}

/// Addresses a value at the RAM address in a register with displacement
fn address_register_indirect_with_displacement(
    cpu: &mut CPU<impl crate::ram::Memory>,
    register: Register,
    displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, CPUError> {
    cpu.memory
        .read(cpu.registers.get(register) + displacement, size)
}

/// Addresses a value at the RAM address in a register with indexing
fn address_register_indirect_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    address_register: Register,
    index_register: Register,
    index_scale: u32,
    displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, CPUError> {
    let base_address = cpu.registers.get(address_register);
    let index_value = cpu.registers.get(index_register) * index_scale;
    let operand_address = base_address + displacement + index_value;

    match size {
        OperandSize::Byte => Ok(M68kInteger::Byte(cpu.memory.read_byte(operand_address)?)),
        OperandSize::Word => Ok(M68kInteger::Word(cpu.memory.read_word(operand_address)?)),
        OperandSize::Long => Ok(M68kInteger::Long(cpu.memory.read_long(operand_address)?)),
    }
}

/// Addresses a value at a given address with a postindex register
fn address_ram_post_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    base_address: u32,
    index_register: Register,
    index_scale: u32,
    base_displacement: u32,
    outer_displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, CPUError> {
    let index_value = cpu.registers.get(index_register) * index_scale;
    let intermediate_address = base_address + base_displacement;
    let intermediate_address_value = cpu.memory.read_long(intermediate_address)?;

    cpu.memory.read(
        intermediate_address_value + index_value + outer_displacement,
        size,
    )
}

/// Addresses a value at a given address with a preindex register
fn address_ram_pre_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    base_address: u32,
    index_register: Register,
    index_scale: u32,
    base_displacement: u32,
    outer_displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, CPUError> {
    let index_value = cpu.registers.get(index_register) * index_scale;
    let intermediate_address = base_address + base_displacement + index_value;
    let intermediate_address_value = cpu.memory.read_long(intermediate_address)?;

    cpu.memory
        .read(intermediate_address_value + outer_displacement, size)
}

impl AddressMode {
    // TODO: does this need to return more than a u32? Should it support getting bytes/words?
    // (probably, but I'd like to see a basic long-only implementation first, to keep myself motivated)
    pub fn get_value(
        &self,
        cpu: &mut CPU<impl crate::ram::Memory>,
    ) -> Result<M68kInteger, CPUError> {
        match *self {
            // Absolute
            AddressMode::Absolute { address, size } => cpu.memory.read(address, size),

            // Immediate
            AddressMode::Immediate { value, size } => match size {
                OperandSize::Byte => Ok(M68kInteger::Byte(value as u8)),
                OperandSize::Word => Ok(M68kInteger::Word(value as u16)),
                OperandSize::Long => Ok(M68kInteger::Long(value)),
            },

            // Register
            AddressMode::RegisterDirect { register, size } => match size {
                OperandSize::Byte => Ok(M68kInteger::Byte(cpu.registers.get(register) as u8)),
                OperandSize::Word => Ok(M68kInteger::Word(cpu.registers.get(register) as u16)),
                OperandSize::Long => Ok(M68kInteger::Long(cpu.registers.get(register))),
            },
            AddressMode::RegisterIndirect { register, size } => cpu
                .memory
                .read(cpu.registers.get_address_register(register), size),
            AddressMode::RegisterIndirectIndexed {
                displacement,
                address_register,
                index_register,
                index_scale,
                size,
            } => address_register_indirect_indexed(
                cpu,
                Register::Address(address_register),
                index_register,
                index_scale as u32,
                displacement as u32,
                size,
            ),
            AddressMode::RegisterIndirectPostIncrement { register, size } => {
                dbg!(size);
                let address = cpu.registers.get_address_register(register);
                let value = cpu.memory.read(address, size)?;
                dbg!(size);
                cpu.registers
                    .set_address_register(register, address + get_increment(register, size));
                Ok(value)
            }
            AddressMode::RegisterIndirectPreDecrement { register, size } => {
                let address =
                    cpu.registers.get_address_register(register) - get_increment(register, size);
                cpu.registers.set_address_register(register, address);
                cpu.memory.read(address, size)
            }
            AddressMode::RegisterIndirectWithDisplacement {
                register,
                displacement,
                size,
            } => address_register_indirect_with_displacement(
                cpu,
                Register::Address(register),
                displacement as u32,
                size,
            ),

            // Program Counter
            AddressMode::ProgramCounterIndirectWithDisplacement { displacement, size } => {
                address_register_indirect_with_displacement(
                    cpu,
                    Register::ProgramCounter,
                    displacement as u32,
                    size,
                )
            }
            AddressMode::ProgramCounterIndirectWithIndex {
                displacement,
                index_register,
                index_scale,
                size,
            } => address_register_indirect_indexed(
                cpu,
                Register::ProgramCounter,
                index_register,
                index_scale as u32,
                displacement as u32,
                size,
            ),
            AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                base_displacement,
                outer_displacement,
                index_register,
                index_scale,
                size,
            } => address_ram_post_indexed(
                cpu,
                cpu.registers.get(Register::ProgramCounter),
                index_register,
                index_scale as u32,
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),
            AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                base_displacement,
                outer_displacement,
                index_register,
                index_scale,
                size,
            } => address_ram_pre_indexed(
                cpu,
                cpu.registers.get(Register::ProgramCounter),
                index_register,
                index_scale as u32,
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),

            // Memory
            AddressMode::MemoryPostIndexed {
                base_displacement,
                outer_displacement,
                address_register,
                index_register,
                index_scale,
                size,
            } => address_ram_post_indexed(
                cpu,
                cpu.registers.get_address_register(address_register),
                index_register,
                index_scale as u32,
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),
            AddressMode::MemoryPreIndexed {
                base_displacement,
                outer_displacement,
                address_register,
                index_register,
                index_scale,
                size,
            } => address_ram_pre_indexed(
                cpu,
                cpu.registers.get_address_register(address_register),
                index_register,
                index_scale as u32,
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ram::{Memory, VecBackedMemory};

    // This saves us from having to hardcode lots of values or declare the same variables in each test function.
    // It also means that we don't have to worry about typing the wrong value.
    static ADDRESS: u32 = 0x42;
    static DISPLACEMENT: u16 = 0xA3;
    static OUTER_DISPLACEMENT: u16 = 0x1A;
    static INDEX: u32 = 4;
    static ADDRESS_REGISTER: AddressRegister = AddressRegister::A0;
    static DATA_REGISTER: DataRegister = DataRegister::D0;

    /// Runs test closure (|size: OperandSize, value: M68kInteger} { ... })
    fn all_sizes(closure: impl Fn(OperandSize, M68kInteger) -> ()) {
        closure(OperandSize::Byte, M68kInteger::Byte(0xAB));
        closure(OperandSize::Word, M68kInteger::Word(0xDEAD));
        closure(OperandSize::Long, M68kInteger::Long(0xFACEBEEF));
    }

    #[test]
    fn register_direct() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let address = AddressMode::RegisterDirect {
                register: Register::Data(DATA_REGISTER),
                size,
            };

            cpu.registers.set_data_register(DATA_REGISTER, value);
            assert_eq!(address.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn register_indirect() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::RegisterIndirect {
                register: ADDRESS_REGISTER,
                size,
            };

            cpu.memory.write(ADDRESS, value).unwrap();
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn register_indirect_with_postincrement() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let byte_offset = size.size_in_bytes();
            let mode = AddressMode::RegisterIndirectPostIncrement {
                register: ADDRESS_REGISTER,
                size,
            };

            cpu.memory.write(ADDRESS, value).unwrap();
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);

            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS + byte_offset
            );
        });
    }

    #[test]
    fn register_indirect_with_predecrement() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let byte_offset = size.size_in_bytes();
            let mode = AddressMode::RegisterIndirectPreDecrement {
                register: ADDRESS_REGISTER,
                size,
            };

            cpu.memory.write(ADDRESS - byte_offset, value).unwrap();
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);

            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS - byte_offset
            );
        });
    }

    #[test]
    /// "if the address register is stack pointer and operand size is byte,
    /// the address is incremented by 2 to preserve alignment" (Scarpazza)
    fn register_indirect_stack_pointer_special_case() {
        let mut cpu = CPU::<VecBackedMemory>::new(8192);
        let post_incr = AddressMode::RegisterIndirectPostIncrement {
            register: AddressRegister::A7,
            size: OperandSize::Byte,
        };
        cpu.registers
            .set_address_register(AddressRegister::A7, ADDRESS);
        post_incr.get_value(&mut cpu).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::A7),
            ADDRESS + 2
        ); // not +1

        let pre_decr = AddressMode::RegisterIndirectPreDecrement {
            register: AddressRegister::A7,
            size: OperandSize::Byte,
        };
        cpu.registers
            .set_address_register(AddressRegister::A7, ADDRESS);
        pre_decr.get_value(&mut cpu).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::A7),
            ADDRESS - 2
        ); // not -1
    }

    #[test]
    fn register_indirect_with_displacement() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::RegisterIndirectWithDisplacement {
                displacement: DISPLACEMENT,
                register: ADDRESS_REGISTER,
                size,
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            let addr = ADDRESS + DISPLACEMENT as u32;
            cpu.memory.write(addr, value).unwrap();
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn register_indirect_indexed() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::RegisterIndirectIndexed {
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
                size,
                index_scale: IndexScale::Two,
                displacement: DISPLACEMENT,
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);
            cpu.memory
                .write(ADDRESS + DISPLACEMENT as u32 + (INDEX * 2), value)
                .unwrap();

            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn memory_post_indexed() {
        all_sizes(|size, value| {
            let initial_address = 0xAA;
            let mut cpu = CPU::<VecBackedMemory>::new(350);
            let mode = AddressMode::MemoryPostIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
                size,
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32;
            let operand_address = ADDRESS + (INDEX * 4) + OUTER_DISPLACEMENT as u32;
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory.write(operand_address, value).unwrap();
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn memory_pre_indexed() {
        all_sizes(|size, value| {
            let initial_address = 0xAA;
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::MemoryPreIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
                size,
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32 + (INDEX * 4);
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();

            let operand_address = ADDRESS + OUTER_DISPLACEMENT as u32;
            cpu.memory.write(operand_address, value).unwrap();
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn program_counter_indirect_with_displacement() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::ProgramCounterIndirectWithDisplacement {
                displacement: DISPLACEMENT,
                size,
            };

            cpu.registers.set(Register::ProgramCounter, ADDRESS);
            cpu.memory
                .write(ADDRESS + DISPLACEMENT as u32, value)
                .unwrap();
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn program_counter_indirect_indexed() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::ProgramCounterIndirectWithIndex {
                displacement: DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Two,
                size,
            };

            cpu.registers.set(Register::ProgramCounter, ADDRESS);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            cpu.memory
                .write(ADDRESS + DISPLACEMENT as u32 + (INDEX * 2), value)
                .unwrap();
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn program_counter_memory_post_indexed() {
        all_sizes(|size, value| {
            let initial_address = 0xAA;
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
                size,
            };

            cpu.registers.set(Register::ProgramCounter, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32;
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory
                .write(ADDRESS + (INDEX * 4) + OUTER_DISPLACEMENT as u32, value)
                .unwrap();

            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn program_counter_memory_pre_indexed() {
        all_sizes(|size, value| {
            let initial_address = 0xAA;
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
                index_scale: IndexScale::Four,
                size,
            };

            cpu.registers.set(Register::ProgramCounter, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32 + (INDEX * 4);
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            cpu.memory
                .write(ADDRESS + OUTER_DISPLACEMENT as u32, value)
                .unwrap();

            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn absolute() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::Absolute {
                address: ADDRESS,
                size,
            };

            cpu.memory.write(ADDRESS, value).unwrap();
            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }

    #[test]
    fn immediate() {
        all_sizes(|size, value| {
            let mut cpu = CPU::<VecBackedMemory>::new(8192);
            let mode = AddressMode::Immediate {
                value: value.into(),
                size,
            };

            assert_eq!(mode.get_value(&mut cpu).unwrap(), value);
        });
    }
}
