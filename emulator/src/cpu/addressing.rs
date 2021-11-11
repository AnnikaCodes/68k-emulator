//! Emulates the 68000's addressing modes.
//!
//! Refer to http://www.scarpaz.com/Attic/Didattica/Scarpazza-2005-68k-1-addressing.pdf
//! and http://faculty.cs.niu.edu/~winans/CS463/notes/amodes.pdf for reference on how these work.

use m68kdecode::{Indexer, MemoryIndirection};

use crate::ram::Memory;
use crate::{EmulationError, M68kInteger, OperandSize};

use super::{registers::*, CPU};

/// Index register scaling - the ONLY legal values for this are 1, 2, and 4.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum IndexScale {
    One = 1,
    Two = 2,
    Four = 4,
}

// TODO: is this correct? consult the docs!
impl From<OperandSize> for IndexScale {
    fn from(size: OperandSize) -> Self {
        match size {
            OperandSize::Byte => IndexScale::One,
            OperandSize::Word => IndexScale::Two,
            OperandSize::Long => IndexScale::Four,
        }
    }
}

#[derive(Debug, Clone, PartialEq)] // remove if perf issue
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
    },
    RegisterIndirectPreDecrement {
        register: AddressRegister,
    },
    RegisterIndirectWithDisplacement {
        register: AddressRegister,
        displacement: u16,
    },
    RegisterIndirectIndexed {
        displacement: u16,
        address_register: AddressRegister,
        index_register: Register,
    },

    // Memory-based addressing
    MemoryPostIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        address_register: AddressRegister,
        index_register: Register,
    },
    MemoryPreIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        address_register: AddressRegister,
        index_register: Register,
    },

    // Program counter-based addressing
    ProgramCounterIndirectWithDisplacement {
        displacement: u16,
    },
    ProgramCounterIndirectIndexed {
        displacement: u16,
        index_register: Register,
    },
    ProgramCounterMemoryIndirectPreIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        index_register: Register,
    },
    ProgramCounterMemoryIndirectPostIndexed {
        base_displacement: u16,
        outer_displacement: u16,
        index_register: Register,
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

/// Gets the increment for a given register + get_value size
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

/// Gets a value at the RAM address in a register with displacement
fn get_address_register_indirect_with_displacement(
    cpu: &mut CPU<impl crate::ram::Memory>,
    register: Register,
    displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, EmulationError> {
    cpu.memory
        .read(cpu.registers.get(register) + displacement, size)
}

/// Sets a value at the RAM address in a register with displacement
fn set_address_register_indirect_with_displacement(
    cpu: &mut CPU<impl crate::ram::Memory>,
    register: Register,
    displacement: u32,
    value: M68kInteger,
) -> Result<(), EmulationError> {
    cpu.memory
        .write(cpu.registers.get(register) + displacement, value)
}

/// Gets a value at the RAM address in a register with indexing
fn get_address_register_indirect_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    address_register: Register,
    index_register: Register,
    index_scale: u32,
    displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, EmulationError> {
    let base_address = cpu.registers.get(address_register);
    let index_value = cpu.registers.get(index_register) * index_scale;
    let operand_address = base_address + displacement + index_value;

    cpu.memory.read(operand_address, size)
}

/// Sets a value at the RAM address in a register with indexing
fn set_address_register_indirect_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    address_register: Register,
    index_register: Register,
    index_scale: u32,
    displacement: u32,
    value: M68kInteger,
) -> Result<(), EmulationError> {
    let base_address = cpu.registers.get(address_register);
    let index_value = cpu.registers.get(index_register) * index_scale;
    let operand_address = base_address + displacement + index_value;

    cpu.memory.write(operand_address, value)
}

/// Gets a value at a given address with a postindex register
fn get_address_ram_post_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    base_address: u32,
    index_register: Register,
    index_scale: u32,
    base_displacement: u32,
    outer_displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, EmulationError> {
    let index_value = cpu.registers.get(index_register) * index_scale;
    let intermediate_address = base_address + base_displacement;
    let intermediate_address_value = cpu.memory.read_long(intermediate_address)?;
    cpu.memory.read(
        intermediate_address_value + index_value + outer_displacement,
        size,
    )
}
/// Gets a value at a given address with a postindex register
fn set_address_ram_post_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    base_address: u32,
    index_register: Register,
    index_scale: u32,
    base_displacement: u32,
    outer_displacement: u32,
    value: M68kInteger,
) -> Result<(), EmulationError> {
    let index_value = cpu.registers.get(index_register) * index_scale;
    let intermediate_address = base_address + base_displacement;
    let intermediate_address_value = cpu.memory.read_long(intermediate_address)?;

    cpu.memory.write(
        intermediate_address_value + index_value + outer_displacement,
        value,
    )
}

/// Gets a value at a given address with a preindex register
fn get_address_ram_pre_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    base_address: u32,
    index_register: Register,
    index_scale: u32,
    base_displacement: u32,
    outer_displacement: u32,
    size: OperandSize,
) -> Result<M68kInteger, EmulationError> {
    let index_value = cpu.registers.get(index_register) * index_scale;
    let intermediate_address = base_address + base_displacement + index_value;
    let intermediate_address_value = cpu.memory.read_long(intermediate_address)?;

    cpu.memory
        .read(intermediate_address_value + outer_displacement, size)
}

/// Sets a value at a given address with a preindex register
fn set_address_ram_pre_indexed(
    cpu: &mut CPU<impl crate::ram::Memory>,
    base_address: u32,
    index_register: Register,
    index_scale: u32,
    base_displacement: u32,
    outer_displacement: u32,
    value: M68kInteger,
) -> Result<(), EmulationError> {
    let index_value = cpu.registers.get(index_register) * index_scale;
    let intermediate_address = base_address + base_displacement + index_value;
    let intermediate_address_value = cpu.memory.read_long(intermediate_address)?;

    cpu.memory
        .write(intermediate_address_value + outer_displacement, value)
}

impl AddressMode {
    /// Converts an m68kdecode instruction to a (src, dest) pair of AddressModes
    ///
    /// TODO: refactor m68kdecode to use my types natively, or use its types in this pro
    pub fn from_m68kdecode(
        source: m68kdecode::Operand,
        destination: m68kdecode::Operand,
    ) -> Result<(AddressMode, Option<AddressMode>), EmulationError> {
        Ok((
            // Should be OK to unwrap since we won't have 2 NoOperands
            AddressMode::from_m68kdecode_operand(source).unwrap(),
            AddressMode::from_m68kdecode_operand(destination),
        ))
    }

    fn from_m68kdecode_operand(op: m68kdecode::Operand) -> Option<AddressMode> {
        match op {
            m68kdecode::Operand::IMM8(value) => Some(AddressMode::Immediate {
                value: value.into(),
            }),
            m68kdecode::Operand::IMM16(value) => Some(AddressMode::Immediate {
                value: value.into(),
            }),
            m68kdecode::Operand::IMM32(value) => Some(AddressMode::Immediate { value }),

            m68kdecode::Operand::ABS16(address) => Some(AddressMode::Absolute {
                address: address as u32,
            }),
            m68kdecode::Operand::ABS32(address) => Some(AddressMode::Absolute { address }),

            m68kdecode::Operand::DR(reg) => Some(AddressMode::RegisterDirect {
                register: Register::Data(reg.into()),
            }),
            m68kdecode::Operand::AR(reg) => Some(AddressMode::RegisterDirect {
                register: Register::Address(reg.into()),
            }),
            m68kdecode::Operand::ARIND(reg) => Some(AddressMode::RegisterIndirect {
                register: reg.into(),
            }),
            m68kdecode::Operand::ARINC(reg) => Some(AddressMode::RegisterIndirectPostIncrement {
                register: reg.into(),
            }),
            m68kdecode::Operand::ARDEC(reg) => Some(AddressMode::RegisterIndirectPreDecrement {
                register: reg.into(),
            }),
            m68kdecode::Operand::ARDISP(reg, disp) => {
                // This is gross! TODO: refactor either us or m68kdecode to be better
                match disp.indexer {
                    Indexer::AR(index_reg, offset) => {
                        Some(Self::from_m68kdecode_with_register_indexing(
                            disp.indirection,
                            Register::Address(reg.into()),
                            Register::Address(index_reg.into()),
                            offset,
                            disp.base_displacement as u16,
                            disp.outer_displacement as u16,
                        ))
                    }
                    Indexer::DR(index_reg, offset) => {
                        Some(Self::from_m68kdecode_with_register_indexing(
                            disp.indirection,
                            Register::Address(reg.into()),
                            Register::Data(index_reg.into()),
                            offset,
                            disp.base_displacement as u16,
                            disp.outer_displacement as u16,
                        ))
                    }
                    Indexer::NoIndexer => match disp.indirection {
                        MemoryIndirection::Indirect
                        | MemoryIndirection::IndirectPostIndexed
                        | MemoryIndirection::IndirectPreIndexed => {
                            panic!("Should not have memory indirect addressing without indexing")
                        }
                        MemoryIndirection::NoIndirection => {
                            Some(AddressMode::RegisterIndirectWithDisplacement {
                                register: reg.into(),
                                displacement: disp.base_displacement as u16,
                            })
                        }
                    },
                }
            }
            m68kdecode::Operand::PCDISP(_size, disp) => {
                // This is gross! TODO: refactor either us or m68kdecode to be better
                match disp.indexer {
                    Indexer::AR(index_reg, offset) => {
                        // TODO: set `size` to be size
                        Some(Self::from_m68kdecode_with_register_indexing(
                            disp.indirection,
                            Register::ProgramCounter,
                            Register::Address(index_reg.into()),
                            offset,
                            disp.base_displacement as u16,
                            disp.outer_displacement as u16,
                        ))
                    }
                    Indexer::DR(index_reg, offset) => {
                        Some(Self::from_m68kdecode_with_register_indexing(
                            disp.indirection,
                            Register::ProgramCounter,
                            Register::Data(index_reg.into()),
                            offset,
                            disp.base_displacement as u16,
                            disp.outer_displacement as u16,
                        ))
                    }
                    Indexer::NoIndexer => match disp.indirection {
                        MemoryIndirection::Indirect
                        | MemoryIndirection::IndirectPostIndexed
                        | MemoryIndirection::IndirectPreIndexed => {
                            panic!("Should not have memory indirect addressing without indexing")
                        }
                        MemoryIndirection::NoIndirection => {
                            Some(AddressMode::ProgramCounterIndirectWithDisplacement {
                                displacement: disp.base_displacement as u16,
                            })
                        }
                    },
                }
            }
            m68kdecode::Operand::NoOperand => None,

            _ => unimplemented!("converting m68kdecode operand {:?} to AddressMode", op),
        }
    }

    fn from_m68kdecode_with_register_indexing(
        indirection: MemoryIndirection,
        address_register: Register,
        index_register: Register,
        _offset: u8,
        base_displacement: u16,
        outer_displacement: u16,
    ) -> Self {
        match indirection {
            MemoryIndirection::Indirect => {
                match address_register {
                    Register::ProgramCounter => AddressMode::ProgramCounterIndirectIndexed {
                        displacement: base_displacement,
                        index_register,
                    },
                    _ => panic!("Shouldn't have memory indirect indexing without pre/postindexing specified except for PC")
                }
            }
            // TODO: handle offset
            MemoryIndirection::IndirectPostIndexed => match address_register {
                Register::Address(ar) => AddressMode::MemoryPostIndexed {
                    address_register: ar,
                    index_register,
                    base_displacement,
                    outer_displacement,
                },
                Register::ProgramCounter => AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                    index_register,
                    base_displacement,
                    outer_displacement,
                },
                Register::Data(_) => unimplemented!("data register with postindexed addressing"),
            }
            MemoryIndirection::IndirectPreIndexed =>  match address_register {
                Register::Address(ar) => AddressMode::MemoryPreIndexed {
                    address_register: ar,
                    index_register,
                    base_displacement,
                    outer_displacement,
                },
                Register::ProgramCounter => AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                    index_register,
                    base_displacement,
                    outer_displacement,
                },
                Register::Data(_) => unimplemented!("data register with preindexed addressing"),
            }
            MemoryIndirection::NoIndirection => match address_register {
                Register::Address(ar) => AddressMode::RegisterIndirectWithDisplacement {
                    register: ar,
                    displacement: base_displacement,
                },
                Register::ProgramCounter => AddressMode::ProgramCounterIndirectWithDisplacement {
                    displacement: base_displacement,
                },
                Register::Data(_) => unimplemented!("data register where address/PC expected"),
            }
        }
    }

    /// Gets the value referenced by this address
    ///
    /// Should return the same size `M68kInteger` as the `OperandSize` given in the enum
    pub fn get_value(
        &self,
        cpu: &mut CPU<impl crate::ram::Memory>,
        size: OperandSize,
    ) -> Result<M68kInteger, EmulationError> {
        match *self {
            // Absolute
            AddressMode::Absolute { address } => cpu.memory.read(address, size),

            // Immediate
            AddressMode::Immediate { value } => match size {
                OperandSize::Byte => Ok(M68kInteger::Byte(value as u8)),
                OperandSize::Word => Ok(M68kInteger::Word(value as u16)),
                OperandSize::Long => Ok(M68kInteger::Long(value)),
            },

            // Register
            AddressMode::RegisterDirect { register } => match size {
                OperandSize::Byte => Ok(M68kInteger::Byte(cpu.registers.get(register) as u8)),
                OperandSize::Word => Ok(M68kInteger::Word(cpu.registers.get(register) as u16)),
                OperandSize::Long => Ok(M68kInteger::Long(cpu.registers.get(register))),
            },
            AddressMode::RegisterIndirect { register } => cpu
                .memory
                .read(cpu.registers.get_address_register(register), size),
            AddressMode::RegisterIndirectIndexed {
                displacement,
                address_register,
                index_register,
            } => get_address_register_indirect_indexed(
                cpu,
                Register::Address(address_register),
                index_register,
                size.size_in_bytes(),
                displacement as u32,
                size,
            ),
            AddressMode::RegisterIndirectPostIncrement { register } => {
                let address = cpu.registers.get_address_register(register);
                let value = cpu.memory.read(address, size)?;
                cpu.registers
                    .set_address_register(register, address + get_increment(register, size));
                Ok(value)
            }
            AddressMode::RegisterIndirectPreDecrement { register } => {
                let address =
                    cpu.registers.get_address_register(register) - get_increment(register, size);
                cpu.registers.set_address_register(register, address);
                cpu.memory.read(address, size)
            }
            AddressMode::RegisterIndirectWithDisplacement {
                register,
                displacement,
            } => get_address_register_indirect_with_displacement(
                cpu,
                Register::Address(register),
                displacement as u32,
                size,
            ),

            // Program Counter
            AddressMode::ProgramCounterIndirectWithDisplacement { displacement } => {
                get_address_register_indirect_with_displacement(
                    cpu,
                    Register::ProgramCounter,
                    displacement as u32,
                    size,
                )
            }
            AddressMode::ProgramCounterIndirectIndexed {
                displacement,
                index_register,
            } => get_address_register_indirect_indexed(
                cpu,
                Register::ProgramCounter,
                index_register,
                size.size_in_bytes(),
                displacement as u32,
                size,
            ),
            AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                base_displacement,
                outer_displacement,
                index_register,
            } => get_address_ram_post_indexed(
                cpu,
                cpu.registers.get(Register::ProgramCounter),
                index_register,
                size.size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),
            AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                base_displacement,
                outer_displacement,
                index_register,
            } => get_address_ram_pre_indexed(
                cpu,
                cpu.registers.get(Register::ProgramCounter),
                index_register,
                size.size_in_bytes(),
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
            } => get_address_ram_post_indexed(
                cpu,
                cpu.registers.get_address_register(address_register),
                index_register,
                size.size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),
            AddressMode::MemoryPreIndexed {
                base_displacement,
                outer_displacement,
                address_register,
                index_register,
            } => get_address_ram_pre_indexed(
                cpu,
                cpu.registers.get_address_register(address_register),
                index_register,
                size.size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                size,
            ),
        }
    }

    /// Sets the RAM or register referenced by this address to the given value
    ///
    /// Currently, this also checks that the provided `new_value` is the same size as the `OperandSize` given in the enum;
    /// if this presents a performance issue, we can remove it or make it configurable.
    ///
    /// Also, it currently leads to quite a bit of code repetition; in the future, I might refactor this
    /// to only write the size-checking line once, probably by peeking into the enum
    /// or specifying the OperandSize without wrapping it in an enum.
    pub fn set_value(
        &self,
        cpu: &mut CPU<impl Memory>,
        new_value: M68kInteger,
    ) -> Result<(), EmulationError> {
        match *self {
            // Absolute
            AddressMode::Absolute { address } => cpu.memory.write(address, new_value),

            // Immediate
            AddressMode::Immediate { .. } => Err(EmulationError::WriteToReadOnly(
                "can't write to constant value".to_string(),
            )),

            // Register
            AddressMode::RegisterDirect { register } => {
                let new_value: u32 = new_value.into();
                cpu.registers.set(register, new_value);
                Ok(())
            }

            AddressMode::RegisterIndirect { register } => cpu
                .memory
                .write(cpu.registers.get_address_register(register), new_value),
            AddressMode::RegisterIndirectIndexed {
                displacement,
                address_register,
                index_register,
            } => set_address_register_indirect_indexed(
                cpu,
                Register::Address(address_register),
                index_register,
                new_value.size().size_in_bytes(),
                displacement as u32,
                new_value,
            ),
            AddressMode::RegisterIndirectPostIncrement { register } => {
                let address = cpu.registers.get_address_register(register);
                cpu.memory.write(address, new_value)?;
                cpu.registers.set_address_register(
                    register,
                    address + get_increment(register, new_value.size()),
                );
                Ok(())
            }
            AddressMode::RegisterIndirectPreDecrement { register } => {
                // could optimize this by having an increment/decrement register method
                let address = cpu.registers.get_address_register(register)
                    - get_increment(register, new_value.size());
                cpu.registers.set_address_register(register, address);
                cpu.memory.write(address, new_value)
            }
            AddressMode::RegisterIndirectWithDisplacement {
                register,
                displacement,
            } => set_address_register_indirect_with_displacement(
                cpu,
                Register::Address(register),
                displacement as u32,
                new_value,
            ),

            // Program Counter
            AddressMode::ProgramCounterIndirectWithDisplacement { displacement } => {
                set_address_register_indirect_with_displacement(
                    cpu,
                    Register::ProgramCounter,
                    displacement as u32,
                    new_value,
                )
            }
            AddressMode::ProgramCounterIndirectIndexed {
                displacement,
                index_register,
            } => set_address_register_indirect_indexed(
                cpu,
                Register::ProgramCounter,
                index_register,
                new_value.size().size_in_bytes(),
                displacement as u32,
                new_value,
            ),
            AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                base_displacement,
                outer_displacement,
                index_register,
            } => set_address_ram_post_indexed(
                cpu,
                cpu.registers.get(Register::ProgramCounter),
                index_register,
                new_value.size().size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                new_value,
            ),
            AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                base_displacement,
                outer_displacement,
                index_register,
            } => set_address_ram_pre_indexed(
                cpu,
                cpu.registers.get(Register::ProgramCounter),
                index_register,
                new_value.size().size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                new_value,
            ),

            // Memory
            AddressMode::MemoryPostIndexed {
                base_displacement,
                outer_displacement,
                address_register,
                index_register,
            } => set_address_ram_post_indexed(
                cpu,
                cpu.registers.get_address_register(address_register),
                index_register,
                new_value.size().size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                new_value,
            ),
            AddressMode::MemoryPreIndexed {
                base_displacement,
                outer_displacement,
                address_register,
                index_register,
            } => set_address_ram_pre_indexed(
                cpu,
                cpu.registers.get_address_register(address_register),
                index_register,
                new_value.size().size_in_bytes(),
                base_displacement as u32,
                outer_displacement as u32,
                new_value,
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

    /// Runs test closure (|size: OperandSize, get_value: M68kInteger, set_value: M68kInteger| { ... })
    fn all_sizes(
        closure: impl Fn(
            CPU<VecBackedMemory>,
            OperandSize,
            M68kInteger,
            M68kInteger,
        ) -> Result<(), EmulationError>,
    ) {
        let cpu1 = CPU::<VecBackedMemory>::new(1_024);
        let cpu2 = CPU::<VecBackedMemory>::new(1_024);
        let cpu3 = CPU::<VecBackedMemory>::new(1_024);

        closure(
            cpu1,
            OperandSize::Byte,
            M68kInteger::Byte(0xAB),
            M68kInteger::Byte(0x73),
        )
        .unwrap();
        closure(
            cpu2,
            OperandSize::Word,
            M68kInteger::Word(0xDEAD),
            M68kInteger::Word(0xABBA),
        )
        .unwrap();
        closure(
            cpu3,
            OperandSize::Long,
            M68kInteger::Long(0xFACEBEEF),
            M68kInteger::Long(0xAF7B3AD),
        )
        .unwrap();
    }

    #[test]
    fn register_direct() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let address = AddressMode::RegisterDirect {
                register: Register::Data(DATA_REGISTER),
            };

            // get
            cpu.registers.set_data_register(DATA_REGISTER, get_value);
            assert_eq!(address.get_value(&mut cpu, size)?, get_value);

            // set
            address.set_value(&mut cpu, set_value)?;
            assert_eq!(
                cpu.registers.get_data_register(DATA_REGISTER),
                set_value.into()
            );
            Ok(())
        });
    }

    #[test]
    fn register_indirect() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let mode = AddressMode::RegisterIndirect {
                register: ADDRESS_REGISTER,
            };

            cpu.memory.write(ADDRESS, get_value)?;
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(ADDRESS, size)?, set_value);
            Ok(())
        });
    }

    #[test]
    fn register_indirect_with_postincrement() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let byte_offset = size.size_in_bytes();
            let mode = AddressMode::RegisterIndirectPostIncrement {
                register: ADDRESS_REGISTER,
            };

            // get
            cpu.memory.write(ADDRESS, get_value)?;
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS + byte_offset
            );

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(ADDRESS + byte_offset, size)?, set_value);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS + (byte_offset * 2)
            );
            Ok(())
        });
    }

    #[test]
    fn register_indirect_with_predecrement() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let byte_offset = size.size_in_bytes();
            let mode = AddressMode::RegisterIndirectPreDecrement {
                register: ADDRESS_REGISTER,
            };

            // get
            cpu.memory.write(ADDRESS - byte_offset, get_value).unwrap();
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            assert_eq!(mode.get_value(&mut cpu, size).unwrap(), get_value);
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS - byte_offset
            );

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(
                cpu.memory.read(ADDRESS - (byte_offset * 2), size)?,
                set_value
            );
            assert_eq!(
                cpu.registers.get_address_register(ADDRESS_REGISTER),
                ADDRESS - (byte_offset * 2)
            );
            Ok(())
        });
    }

    #[test]
    /// "if the address register is stack pointer and operand size is byte,
    /// the address is incremented by 2 to preserve alignment" (Scarpazza)
    fn register_indirect_stack_pointer_special_case() {
        let mut cpu = CPU::<VecBackedMemory>::new(8192);
        let post_incr = AddressMode::RegisterIndirectPostIncrement {
            register: AddressRegister::A7,
        };

        // get
        cpu.registers
            .set_address_register(AddressRegister::A7, ADDRESS);
        post_incr.get_value(&mut cpu, OperandSize::Byte).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::A7),
            ADDRESS + 2
        ); // not +1

        // set
        post_incr.set_value(&mut cpu, M68kInteger::Byte(1)).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::A7),
            ADDRESS + 4
        ); // not +2

        let pre_decr = AddressMode::RegisterIndirectPreDecrement {
            register: AddressRegister::A7,
        };

        // get
        cpu.registers
            .set_address_register(AddressRegister::A7, ADDRESS);
        pre_decr.get_value(&mut cpu, OperandSize::Byte).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::A7),
            ADDRESS - 2
        ); // not -1

        // set
        pre_decr.set_value(&mut cpu, M68kInteger::Byte(1)).unwrap();
        assert_eq!(
            cpu.registers.get_address_register(AddressRegister::A7),
            ADDRESS - 4
        ); // not -2
    }

    #[test]
    fn register_indirect_with_displacement() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let mode = AddressMode::RegisterIndirectWithDisplacement {
                displacement: DISPLACEMENT,
                register: ADDRESS_REGISTER,
            };

            // get
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            let addr = ADDRESS + DISPLACEMENT as u32;
            cpu.memory.write(addr, get_value).unwrap();
            assert_eq!(mode.get_value(&mut cpu, size).unwrap(), get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(addr, size)?, set_value);
            Ok(())
        });
    }

    #[test]
    fn register_indirect_indexed() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let mode = AddressMode::RegisterIndirectIndexed {
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
                displacement: DISPLACEMENT,
            };

            let addr = ADDRESS + DISPLACEMENT as u32 + (INDEX * size.size_in_bytes());

            // get
            cpu.registers
                .set_address_register(ADDRESS_REGISTER, ADDRESS);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);
            cpu.memory.write(addr, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(addr, size)?, set_value);
            Ok(())
        });
    }

    #[test]
    fn memory_post_indexed() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let initial_address = 0xAA;
            let mode = AddressMode::MemoryPostIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32;
            let operand_address =
                ADDRESS + (INDEX * size.size_in_bytes()) + OUTER_DISPLACEMENT as u32;
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();

            // get
            cpu.memory.write(operand_address, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(operand_address, size)?, set_value);
            Ok(())
        });
    }

    #[test]
    fn memory_pre_indexed() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let initial_address = 0xAA;
            let mode = AddressMode::MemoryPreIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                address_register: ADDRESS_REGISTER,
                index_register: Register::Data(DATA_REGISTER),
            };

            cpu.registers
                .set_address_register(ADDRESS_REGISTER, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address =
                initial_address + DISPLACEMENT as u32 + (INDEX * size.size_in_bytes());
            cpu.memory.write_long(intermediate_address, ADDRESS)?;
            let operand_address = ADDRESS + OUTER_DISPLACEMENT as u32;

            // get
            cpu.memory.write(operand_address, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(operand_address, size)?, set_value);
            Ok(())
        });
    }

    #[test]
    fn program_counter_indirect_with_displacement() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let mode = AddressMode::ProgramCounterIndirectWithDisplacement {
                displacement: DISPLACEMENT,
            };

            cpu.registers.set(Register::ProgramCounter, ADDRESS);
            let address = ADDRESS + DISPLACEMENT as u32;

            // get
            cpu.memory.write(address, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(address, size)?, set_value);

            Ok(())
        });
    }

    #[test]
    fn program_counter_indirect_indexed() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let mode = AddressMode::ProgramCounterIndirectIndexed {
                displacement: DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
            };

            cpu.registers.set(Register::ProgramCounter, ADDRESS);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let address = ADDRESS + DISPLACEMENT as u32 + (INDEX * size.size_in_bytes());
            cpu.memory.write(address, get_value)?;

            // get
            cpu.memory.write(address, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(address, size)?, set_value);

            Ok(())
        });
    }

    #[test]
    fn program_counter_memory_post_indexed() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let initial_address = 0xAA;
            let mode = AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
            };

            cpu.registers.set(Register::ProgramCounter, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address = initial_address + DISPLACEMENT as u32;
            cpu.memory
                .write_long(intermediate_address, ADDRESS)
                .unwrap();
            let final_address =
                ADDRESS + (INDEX * size.size_in_bytes()) + OUTER_DISPLACEMENT as u32;

            // get
            cpu.memory.write(final_address, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(final_address, size)?, set_value);

            Ok(())
        });
    }

    #[test]
    fn program_counter_memory_pre_indexed() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let initial_address = 0xAA;
            let mode = AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                base_displacement: DISPLACEMENT,
                outer_displacement: OUTER_DISPLACEMENT,
                index_register: Register::Data(DATA_REGISTER),
            };

            cpu.registers.set(Register::ProgramCounter, initial_address);
            cpu.registers.set_data_register(DATA_REGISTER, INDEX);

            let intermediate_address =
                initial_address + DISPLACEMENT as u32 + (INDEX * size.size_in_bytes());
            let final_address = ADDRESS + OUTER_DISPLACEMENT as u32;
            cpu.memory.write_long(intermediate_address, ADDRESS)?;

            // get
            cpu.memory.write(final_address, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(final_address, size)?, set_value);

            Ok(())
        });
    }

    #[test]
    fn absolute() {
        all_sizes(|mut cpu, size, get_value, set_value| {
            let mode = AddressMode::Absolute { address: ADDRESS };

            // get
            cpu.memory.write(ADDRESS, get_value)?;
            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);

            // set
            mode.set_value(&mut cpu, set_value)?;
            assert_eq!(cpu.memory.read(ADDRESS, size)?, set_value);

            Ok(())
        });
    }

    #[test]
    fn immediate() {
        // Immediate doesn't need to set a get_value
        all_sizes(|mut cpu, size, get_value, _| {
            let mode = AddressMode::Immediate {
                value: get_value.into(),
            };

            assert_eq!(mode.get_value(&mut cpu, size)?, get_value);
            Ok(())
        });
    }

    // It makes no sense to set the value in immediate addressing
    #[test]
    #[should_panic]
    fn immediate_panic_on_set() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let mode = AddressMode::Immediate { value: 0xAA };

        mode.set_value(&mut cpu, M68kInteger::Byte(1)).unwrap();
    }
}
