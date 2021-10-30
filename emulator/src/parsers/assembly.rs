//! Parses assembly code

use std::fs::OpenOptions;

use super::{Interpreter, ParseError};
use crate::cpu::{
    addressing::AddressMode,
    isa_68000::*,
    registers::{AddressRegister, DataRegister, Register},
};
use crate::OperandSize;

fn to_u16(int: u32) -> Result<u16, ParseError> {
    match int.try_into() {
        Ok(d) => Ok(d),
        Err(error) => return Err(ParseError::NumberTooLarge(error)),
    }
}

#[derive(Default)]
pub struct AssemblyInterpreter {}

impl AssemblyInterpreter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parses an operand to an address
    ///
    /// TODO: figure out how different operand sizes are represented & handle accordingly in unit tests
    fn parse_to_operand(op_string: &str, instruction: &str) -> Result<AddressMode, ParseError> {
        let mut chars = op_string.chars();
        let first = chars.next();
        match first {
            // Register Direct
            Some('d' | 'a' | 's') => {
                let (register, size) = Self::parse_to_register(op_string)?;
                Ok(AddressMode::RegisterDirect { register, size })
            },
            // Immediate
            Some('#') => Ok(AddressMode::Immediate {
                value: Self::parse_to_number(&op_string[1..])?,
                size: OperandSize::Long,
            }),
            // Indirect Stuff
            Some('(' | '-') => {
                // + is for postincrement
                if !op_string.ends_with(')') && !op_string.ends_with('+') {
                    return Err(ParseError::UnknownOperandFormat {
                        operand: op_string.to_string(),
                        instruction: instruction.to_string(),
                    });
                }

                let op_string = op_string.replace(|c| c == '(' || c == ')', "");
                let mut parts = op_string.split(',').collect::<Vec<&str>>();
                match parts.len() {
                    // Register Indirect potentially with postincr/predecr
                    1 => {
                        let mut register = parts[0];
                        let mut is_postincr = false;
                        let mut is_predecr = false;

                        if register.starts_with('-') {
                            register = &register[1..];
                            is_predecr = true;
                        }
                        if register.ends_with('+') {
                            register = &register[..register.len() - 1];
                            is_postincr = true;
                        }

                        let (register, size) = match Self::parse_to_register(register)? {
                            (Register::Address(reg), size) => (reg, size),
                            _ => {
                                return Err(ParseError::InvalidOperand {
                                    operand: op_string.to_string(),
                                    instruction: instruction.to_string(),
                                })
                            }
                        };

                        if is_postincr && is_predecr {
                            Err(ParseError::UnknownOperandFormat {
                                operand: op_string.to_string(),
                                instruction: instruction.to_string(),
                            })
                        } else if is_postincr {
                            Ok(AddressMode::RegisterIndirectPostIncrement { register, size })
                        } else if is_predecr {
                            Ok(AddressMode::RegisterIndirectPreDecrement { register, size })
                        } else {
                            Ok(AddressMode::RegisterIndirect { register, size })
                        }
                    }
                    // Displacement
                    2 => {
                        let (displacement, register) = (parts[0].trim(), parts[1].trim());
                        let displacement = to_u16(Self::parse_to_number(displacement)?)?;

                        match Self::parse_to_register(register)? {
                            (Register::Address(reg), size) => {
                                Ok(AddressMode::RegisterIndirectWithDisplacement {
                                    displacement,
                                    register: reg,
                                    size,
                                })
                            }
                            (Register::ProgramCounter, size) => {
                                Ok(AddressMode::ProgramCounterIndirectWithDisplacement {
                                    displacement,
                                    size,
                                })
                            }
                            _ => Err(ParseError::InvalidOperand {
                                operand: op_string.to_string(),
                                instruction: instruction.to_string(),
                            }),
                        }
                    }
                    // Indexed indirect addressing
                    3 | 4 if parts[0].starts_with('[') => {
                        let mut for_ia: Vec<&str> = vec![];

                        while let part = parts.remove(0) {
                            if let Some(part) = part.strip_suffix("]") {
                                for_ia.push(part);
                                break;
                            }
                            for_ia.push(part);
                        }

                        let is_preindexed = for_ia.len() == 3;
                        if for_ia.len() < 2 || for_ia.len() > 3 {
                            return Err(ParseError::UnknownOperandFormat {
                                operand: op_string.to_string(),
                                instruction: instruction.to_string(),
                            });
                        }

                        let base_displacement: u16 = to_u16( Self::parse_to_number(for_ia[0].trim_start_matches('['))?)?;
                        let address_register: Register = match Self::parse_to_register(for_ia[1].trim()) {
                            Ok((reg, _)) => reg,
                            _ => return Err(ParseError::InvalidOperand {
                                operand: op_string.to_string(),
                                instruction: instruction.to_string(),
                            })
                        };

                        let idxreg_asm = if is_preindexed {
                            for_ia[2]
                        } else {
                            match parts.get(0) {
                                Some(displacement) => displacement,
                                None => return Err(ParseError::InvalidOperand {
                                    operand: op_string.to_string(),
                                    instruction: instruction.to_string(),
                                }),
                            }
                        };

                        let (index_register, size) = match Self::parse_to_register(idxreg_asm.trim()) {
                            Ok(reg) => reg,
                            _ => return Err(ParseError::InvalidOperand {
                                operand: op_string.to_string(),
                                instruction: instruction.to_string(),
                            })
                        };
                        let outer_displacement = match parts.pop() {
                            Some(displacement) => to_u16(Self::parse_to_number(displacement.trim())?)?,
                            None => 0,
                        };
                        if is_preindexed {
                            match address_register {
                                Register::Address(reg) => Ok(AddressMode::MemoryPreIndexed {
                                    base_displacement,
                                    address_register: reg,
                                    index_register,
                                    index_scale: size.into(),
                                    outer_displacement,
                                    size,
                                }),
                                Register::ProgramCounter => Ok(AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                                    base_displacement,
                                    index_register,
                                    index_scale: size.into(),
                                    outer_displacement,
                                    size,
                                }),
                                _ => Err(ParseError::InvalidOperand {
                                    operand: op_string.to_string(),
                                    instruction: instruction.to_string(),
                                }),
                            }
                        } else {
                            match address_register {
                                Register::Address(reg) => Ok(AddressMode::MemoryPostIndexed {
                                    base_displacement,
                                    address_register: reg,
                                    index_register,
                                    index_scale: size.into(),
                                    outer_displacement,
                                    size,
                                }),
                                Register::ProgramCounter => Ok(AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                                    base_displacement,
                                    index_register,
                                    index_scale: size.into(),
                                    outer_displacement,
                                    size,
                                }),
                                _ => Err(ParseError::InvalidOperand {
                                    operand: op_string.to_string(),
                                    instruction: instruction.to_string(),
                                }),
                            }
                        }
                    }
                    _ => Err(ParseError::UnknownOperandFormat {
                        operand: op_string.to_string(),
                        instruction: instruction.to_string(),
                    }),
                }
            }
            _ => Err(ParseError::UnknownOperandFormat {
                operand: op_string.to_string(),
                instruction: instruction.to_string(),
            }),
        }
    }

    /// Parses a number
    fn parse_to_number(num: &str) -> Result<u32, ParseError> {
        let parse_result = if let Some(hex_num) = num.strip_prefix('$') {
            // Hex
            u32::from_str_radix(hex_num, 16)
        } else {
            num.parse::<u32>()
        };

        match parse_result {
            Ok(num) => Ok(num),
            Err(error) => Err(ParseError::InvalidNumber {
                number: num.to_string(),
                error,
            }),
        }
    }

    /// Parses a string to a register and size
    fn parse_to_register(register: &str) -> Result<(Register, OperandSize), ParseError> {
        if let Some(register) = register.strip_suffix(".b") {
            Ok((Self::parse_to_register_no_size(register)?, OperandSize::Byte))
        } else if let Some(register) = register.strip_suffix(".w") {
            Ok((Self::parse_to_register_no_size(register)?, OperandSize::Word))
        } else if let Some(register) = register.strip_suffix(".l") {
            Ok((Self::parse_to_register_no_size(register)?, OperandSize::Long))
        } else {
            Ok((Self::parse_to_register_no_size(register)?, OperandSize::Long))
        }
    }

    /// Parses a string to a register
    fn parse_to_register_no_size(register: &str) -> Result<Register, ParseError> {
        match register {
            "d0" => Ok(Register::Data(DataRegister::D0)),
            "d1" => Ok(Register::Data(DataRegister::D1)),
            "d2" => Ok(Register::Data(DataRegister::D2)),
            "d3" => Ok(Register::Data(DataRegister::D3)),
            "d4" => Ok(Register::Data(DataRegister::D4)),
            "d5" => Ok(Register::Data(DataRegister::D5)),
            "d6" => Ok(Register::Data(DataRegister::D6)),
            "d7" => Ok(Register::Data(DataRegister::D7)),

            "a0" => Ok(Register::Address(AddressRegister::A0)),
            "a1" => Ok(Register::Address(AddressRegister::A1)),
            "a2" => Ok(Register::Address(AddressRegister::A2)),
            "a3" => Ok(Register::Address(AddressRegister::A3)),
            "a4" => Ok(Register::Address(AddressRegister::A4)),
            "a5" => Ok(Register::Address(AddressRegister::A5)),
            "a6" => Ok(Register::Address(AddressRegister::A6)),
            "a7" | "sp" => Ok(Register::Address(AddressRegister::A7)),

            "pc" => Ok(Register::ProgramCounter),

            _ => Err(ParseError::UnknownRegister(register.to_string())),
        }
    }

    /// Parses source and destination operands
    fn parse_source_dest(
        op_string: &str,
        instruction: String,
    ) -> Result<(AddressMode, AddressMode), ParseError> {
        let mut paren_level: u32 = 0;
        for (idx, token) in op_string.chars().enumerate() {
            match token {
                '(' => paren_level += 1,
                ')' => {
                    if paren_level == 0 {
                        return Err(ParseError::UnexpectedToken { token, instruction });
                    }
                    paren_level -= 1;
                }
                // Commas within parentheticals do NOT demarcate the boundary between source and destination operands;
                // they are part of the assembly representation of certain addressing modes.
                ',' if paren_level == 0 => {
                    let (source_asm, dest_asm) = op_string.split_at(idx);
                    let source = Self::parse_to_operand(source_asm.trim(), &instruction)?;
                    let destination = Self::parse_to_operand(
                        dest_asm.trim_start_matches(|c| c == ' ' || c == ','),
                        &instruction,
                    )?;
                    return Ok((source, destination));
                }
                _ => {}
            }
        }

        Err(ParseError::MissingOperand(instruction))
    }
}

impl Interpreter<String> for AssemblyInterpreter {
    fn parse_instruction(&mut self, source: String) -> Result<ISA68000, ParseError> {
        let lowercase_source = source.to_lowercase();
        let (instruction_token, rest) = match lowercase_source.trim().split_once(' ') {
            Some(s) => s,
            None => return Err(ParseError::NoInstruction(source)),
        };

        match instruction_token {
            "add" => unimplemented!(),
            "sub" => unimplemented!(),
            "move" => {
                let (source_addr, destination_addr) =
                    AssemblyInterpreter::parse_source_dest(rest, source)?;
                Ok(ISA68000::Move {
                    src: source_addr,
                    dest: destination_addr,
                })
            }
            _ => Err(ParseError::UnknownInstruction(
                instruction_token.to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cpu::registers::{AddressRegister, DataRegister, Register::*},
        OperandSize::*,
    };
    use lazy_static::lazy_static;

    lazy_static! {
        static ref DUMMY_INSTRUCTION: String = String::from("Test instruction");
    }

    #[test]
    fn parse_move() {
        for (raw, parsed) in [
            (
                "MOVE a0, a1",
                ISA68000::Move {
                    src: AddressMode::RegisterDirect {
                        register: Address(AddressRegister::A0),
                        size: Long,
                    },
                    dest: AddressMode::RegisterDirect {
                        register: Address(AddressRegister::A1),
                        size: Long,
                    },
                },
            ),
            (
                "MOVE (12, a5), d3",
                ISA68000::Move {
                    src: AddressMode::RegisterIndirectWithDisplacement {
                        register: AddressRegister::A5,
                        displacement: 12,
                        size: Long,
                    },
                    dest: AddressMode::RegisterDirect {
                        register: Data(DataRegister::D3),
                        size: Long,
                    },
                },
            ),
        ] {
            let mut interpreter = AssemblyInterpreter::new();
            assert_eq!(
                interpreter.parse_instruction(raw.to_string()).unwrap(),
                parsed
            );
        }
    }

    #[test]
    fn parse_to_operand_register_direct() {
        for (operand, register) in [
            ("d0", Data(DataRegister::D0)),
            ("a6", Address(AddressRegister::A6)),
            ("a7", Address(AddressRegister::A7)),
            ("sp", Address(AddressRegister::A7)),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::RegisterDirect {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_register_indirect() {
        for (operand, register) in [
            ("(a3)", AddressRegister::A3),
            ("(a7)", AddressRegister::A7),
            ("(sp)", AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::RegisterIndirect {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_register_indirect_postincrement() {
        for (operand, register) in [
            ("(a1)+", AddressRegister::A1),
            ("(a7)+", AddressRegister::A7),
            ("(sp)+", AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::RegisterIndirectPostIncrement {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_register_indirect_predecrement() {
        for (operand, register) in [
            ("-(a5)", AddressRegister::A5),
            ("-(a7)", AddressRegister::A7),
            ("-(sp)", AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::RegisterIndirectPreDecrement {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_register_indirect_displacement() {
        for (operand, displacement, register) in [
            ("(1, a3)", 1, AddressRegister::A3),
            ("(8, a7)", 8, AddressRegister::A7),
            ("(952, sp)", 952, AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::RegisterIndirectWithDisplacement {
                    register,
                    displacement,
                    size: Long,
                }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_register_indirect_indexed() {
        for (operand, displacement, address_register, index_register, size) in [
            (
                "(1, a1, d3.b)",
                1,
                AddressRegister::A1,
                Data(DataRegister::D3),
                Byte,
            ),
            (
                "(8, a7, a4.w)",
                8,
                AddressRegister::A7,
                Address(AddressRegister::A4),
                Word,
            ),
            (
                "(952, sp, d5.l)",
                952,
                AddressRegister::A7,
                Data(DataRegister::D5),
                Long,
            ),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::RegisterIndirectIndexed {
                    address_register,
                    index_register,
                    index_scale: size.into(),
                    displacement,
                    size,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_memory_postindexed() {
        for (
            operand,
            base_displacement,
            outer_displacement,
            address_register,
            index_register,
            size,
        ) in [
            (
                "([1,a1], d3.b, 2)", // TODO: does this need + signs
                1,
                2,
                AddressRegister::A1,
                Data(DataRegister::D3),
                Byte,
            ),
            (
                "([8,a7], a4.w, 952)",
                8,
                952,
                AddressRegister::A7,
                Address(AddressRegister::A4),
                Word,
            ),
            (
                "([952,sp], d5.l, 1)",
                952,
                1,
                AddressRegister::A7,
                Data(DataRegister::D5),
                Long,
            ),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::MemoryPostIndexed {
                    address_register,
                    index_register,
                    index_scale: size.into(),
                    base_displacement,
                    outer_displacement,
                    size,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_memory_preindexed() {
        for (
            operand,
            base_displacement,
            outer_displacement,
            address_register,
            index_register,
            size,
        ) in [
            (
                "([1,a1,d3.b], 2)",
                1,
                2,
                AddressRegister::A1,
                Data(DataRegister::D3),
                Byte,
            ),
            (
                "([8,a7, a4.w],952)",
                8,
                952,
                AddressRegister::A7,
                Address(AddressRegister::A4),
                Word,
            ),
            (
                "([952, sp, d5.l], 1)",
                952,
                1,
                AddressRegister::A7,
                Data(DataRegister::D5),
                Long,
            ),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::MemoryPreIndexed {
                    address_register,
                    index_register,
                    index_scale: size.into(),
                    base_displacement,
                    outer_displacement,
                    size,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_pc_indirect_with_displacement() {
        for (operand, displacement, size) in [
            ("(1, pc)", 1, Long),
            ("(8, pc)", 8, Long),
            ("(952, pc)", 952, Long),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::ProgramCounterIndirectWithDisplacement { displacement, size }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_pc_indirect_indexed() {
        for (operand, displacement, index_register, size) in [
            ("(1, pc, d3.b)", 1, Data(DataRegister::D3), Byte),
            ("(8, pc, a4.w)", 8, Address(AddressRegister::A4), Word),
            ("(952, pc, d5.l)", 952, Data(DataRegister::D5), Long),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::ProgramCounterIndirectWithIndex {
                    index_register,
                    index_scale: size.into(),
                    displacement,
                    size,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_pc_indirect_postindexed() {
        for (operand, base_displacement, outer_displacement, index_register, size) in [
            ("([1,pc], d3.b, 2)", 1, 2, Data(DataRegister::D3), Byte),
            (
                "([8, pc], a4.w, 952)",
                8,
                952,
                Address(AddressRegister::A4),
                Word,
            ),
            ("([952,pc], d5.l,1)", 952, 1, Data(DataRegister::D5), Long),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                    index_register,
                    index_scale: size.into(),
                    base_displacement,
                    outer_displacement,
                    size,
                }
            );
        }
    }

    #[test]
    fn parse_to_operand_pc_preindexed() {
        for (operand, base_displacement, outer_displacement, index_register, size) in [
            ("([1,pc,d3.b], 2)", 1, 2, Data(DataRegister::D3), Byte),
            (
                "([8,pc, a4.w],952)",
                8,
                952,
                Address(AddressRegister::A4),
                Word,
            ),
            ("([952, pc, d5.l], 1)", 952, 1, Data(DataRegister::D5), Long),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                    index_register,
                    index_scale: size.into(),
                    base_displacement,
                    outer_displacement,
                    size,
                }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_absolute() {
        for (operand, address, size) in [("($400).w", 0x400, Word), ("($b4a).l", 0xB4A, Long)] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::Absolute { address, size }
            );
        }
    }

    #[test]
    fn parse_to_operand_immediate() {
        // TODO: should this be a byte, word, or long?
        for (operand, value, size) in [("#$400", 0x400, Long)] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                AddressMode::Immediate { value, size }
            );
        }
    }
}
