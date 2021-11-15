//! Parses assembly code

use super::{ParseError, Parser};
use crate::cpu::{
    addressing::AddressMode,
    isa_68000::*,
    registers::{AddressRegister, DataRegister, Register},
};
use crate::OperandSize;

fn to_u16(int: u32) -> Result<u16, ParseError> {
    match int.try_into() {
        Ok(d) => Ok(d),
        Err(error) => Err(ParseError::NumberTooLarge(error)),
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
    fn parse_to_operand(
        op_string: &str,
        instruction: &str,
    ) -> Result<(AddressMode, Option<OperandSize>), ParseError> {
        let mut chars = op_string.chars();
        let first = chars.next();
        match first {
            // Register Direct
            Some('d' | 'a' | 's') => {
                let (register, size) = Self::parse_to_register(op_string)?;
                Ok((AddressMode::RegisterDirect { register }, size))
            }
            // Immediate
            Some('#') => Ok((
                AddressMode::Immediate {
                    value: Self::parse_to_number(&op_string[1..])?,
                },
                None,
            )),
            // Indirect Stuff + Absolute
            Some('(' | '-') => {
                // Absolute
                if !op_string.contains(',') {
                    // if it includes a comma, it's not an absolute address
                    if let Some('$' | '0'..='9') = chars.next() {
                        let (address_asm, size) = Self::parse_size_suffix(op_string)?;
                        let address = Self::parse_to_number(
                            &address_asm.replace(|c| c == '(' || c == ')', ""),
                        )?;
                        return Ok((AddressMode::Absolute { address }, size));
                    }
                }

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
                            Ok((
                                AddressMode::RegisterIndirectPostIncrement { register },
                                size,
                            ))
                        } else if is_predecr {
                            Ok((AddressMode::RegisterIndirectPreDecrement { register }, size))
                        } else {
                            Ok((AddressMode::RegisterIndirect { register }, size))
                        }
                    }
                    // Displacement
                    2 => {
                        let (displacement, register) = (parts[0].trim(), parts[1].trim());
                        let displacement = to_u16(Self::parse_to_number(displacement)?)?;

                        match Self::parse_to_register(register)? {
                            (Register::Address(reg), size) => Ok((
                                AddressMode::RegisterIndirectWithDisplacement {
                                    displacement,
                                    register: reg,
                                },
                                size,
                            )),
                            (Register::ProgramCounter, size) => Ok((
                                AddressMode::ProgramCounterIndirectWithDisplacement {
                                    displacement,
                                },
                                size,
                            )),
                            _ => Err(ParseError::InvalidOperand {
                                operand: op_string.to_string(),
                                instruction: instruction.to_string(),
                            }),
                        }
                    }
                    // Register/PC indirect with index
                    3 if !parts[0].starts_with('[') => {
                        let displacement = to_u16(Self::parse_to_number(parts[0].trim())?)?;
                        let address_register = Self::parse_to_register_no_size(parts[1].trim())?;
                        let (index_register, size) = Self::parse_to_register(parts[2].trim())?;

                        match address_register {
                            Register::Address(reg) => Ok((
                                AddressMode::RegisterIndirectIndexed {
                                    displacement,
                                    address_register: reg,
                                    index_register,
                                },
                                size,
                            )),
                            Register::ProgramCounter => Ok((
                                AddressMode::ProgramCounterIndirectIndexed {
                                    displacement,
                                    index_register,
                                },
                                size,
                            )),
                            _ => Err(ParseError::InvalidRegister {
                                register: parts[1].to_string(),
                                instruction: instruction.to_string(),
                                reason: String::from(
                                    "Expected address register or program counter",
                                ),
                            }),
                        }
                    }
                    // Memory indexed indirect addressing with post/preindex
                    3 | 4 if parts[0].starts_with('[') => {
                        let mut for_ia: Vec<&str> = vec![];

                        loop {
                            let part = parts.remove(0);
                            if let Some(part) = part.strip_suffix(']') {
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

                        let base_displacement: u16 =
                            to_u16(Self::parse_to_number(for_ia[0].trim_start_matches('['))?)?;
                        let address_register: Register =
                            match Self::parse_to_register(for_ia[1].trim()) {
                                Ok((reg, _)) => reg,
                                _ => {
                                    return Err(ParseError::InvalidOperand {
                                        operand: op_string.to_string(),
                                        instruction: instruction.to_string(),
                                    })
                                }
                            };

                        let idxreg_asm = if is_preindexed {
                            for_ia[2]
                        } else {
                            match parts.get(0) {
                                Some(displacement) => displacement,
                                None => {
                                    return Err(ParseError::InvalidOperand {
                                        operand: op_string.to_string(),
                                        instruction: instruction.to_string(),
                                    })
                                }
                            }
                        };

                        let (index_register, size) =
                            match Self::parse_to_register(idxreg_asm.trim()) {
                                Ok(reg) => reg,
                                _ => {
                                    return Err(ParseError::InvalidOperand {
                                        operand: op_string.to_string(),
                                        instruction: instruction.to_string(),
                                    })
                                }
                            };
                        let outer_displacement = match parts.pop() {
                            Some(displacement) => {
                                to_u16(Self::parse_to_number(displacement.trim())?)?
                            }
                            None => 0,
                        };
                        if is_preindexed {
                            match address_register {
                                Register::Address(reg) => Ok((
                                    AddressMode::MemoryPreIndexed {
                                        base_displacement,
                                        address_register: reg,
                                        index_register,
                                        outer_displacement,
                                    },
                                    size,
                                )),
                                Register::ProgramCounter => Ok((
                                    AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                                        base_displacement,
                                        index_register,
                                        outer_displacement,
                                    },
                                    size,
                                )),
                                _ => Err(ParseError::InvalidOperand {
                                    operand: op_string.to_string(),
                                    instruction: instruction.to_string(),
                                }),
                            }
                        } else {
                            match address_register {
                                Register::Address(reg) => Ok((
                                    AddressMode::MemoryPostIndexed {
                                        base_displacement,
                                        address_register: reg,
                                        index_register,
                                        outer_displacement,
                                    },
                                    size,
                                )),
                                Register::ProgramCounter => Ok((
                                    AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                                        base_displacement,
                                        index_register,
                                        outer_displacement,
                                    },
                                    size,
                                )),
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
    fn parse_to_register(register: &str) -> Result<(Register, Option<OperandSize>), ParseError> {
        let (reg, size) = Self::parse_size_suffix(register)?;
        Ok((Self::parse_to_register_no_size(reg)?, size))
    }

    /// Gets a size suffix
    fn parse_size_suffix(operand: &str) -> Result<(&str, Option<OperandSize>), ParseError> {
        if let Some(operand) = operand.strip_suffix(".b") {
            Ok((operand, Some(OperandSize::Byte)))
        } else if let Some(operand) = operand.strip_suffix(".w") {
            Ok((operand, Some(OperandSize::Word)))
        } else if let Some(operand) = operand.strip_suffix(".l") {
            Ok((operand, Some(OperandSize::Long)))
        } else {
            Ok((operand, None))
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
    ) -> Result<(AddressMode, AddressMode, Option<OperandSize>), ParseError> {
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
                    let (source_mode, source_size) =
                        Self::parse_to_operand(source_asm.trim(), &instruction)?;
                    let (dest_mode, dest_size) = Self::parse_to_operand(
                        dest_asm.trim_start_matches(|c| c == ' ' || c == ','),
                        &instruction,
                    )?;

                    // Please, Rust, stabilize multiple `if let`s in one statement :(
                    if source_size.is_some() && dest_size.is_some() && source_size != dest_size {
                        return Err(ParseError::OperandSizeMismatch {
                            instruction,
                            source_size: source_size.unwrap(),
                            dest_size: dest_size.unwrap(),
                        });
                    }

                    let size = match source_size {
                        Some(size) => Some(size),
                        None => dest_size,
                    };

                    return Ok((source_mode, dest_mode, size));
                }
                _ => {}
            }
        }

        Err(ParseError::MissingOperand(instruction))
    }
}

impl Parser<String> for AssemblyInterpreter {
    fn parse(&mut self, source: String) -> Result<(Instruction, OperandSize, u32), ParseError> {
        let lowercase_source = source.to_lowercase();
        let (instruction_token, rest) = match lowercase_source.trim().split_once(' ') {
            Some(s) => s,
            None => return Err(ParseError::NoInstruction(source)),
        };
        let (src, dest, size) = Self::parse_source_dest(rest, source)?;
        let size = size.unwrap_or(OperandSize::Long);
        match instruction_token {
            "add" => Ok((Instruction::Add { src, dest }, size, 0)),
            "sub" => Ok((Instruction::Subtract { src, dest }, size, 0)),
            "mulu" => Ok((Instruction::MultiplyUnsigned { src, dest }, size, 0)),
            "move" => Ok((Instruction::Move { src, dest }, size, 0)),
            "roxl" => Ok((
                Instruction::RotateLeft {
                    to_rotate: src,
                    rotate_amount: dest,
                },
                size,
                0,
            )),
            "eor" => Ok((Instruction::ExclusiveOr { src, dest }, size, 0)),
            "or" => Ok((Instruction::InclusiveOr { src, dest }, size, 0)),
            "nop" => Ok((Instruction::NoOp, size, 0)),
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

    /// Tests a source/desination instruction.
    ///
    /// gen_instruction is a closure of the form |src, dest| -> Instruction
    ///
    /// For example:
    /// ```
    /// test_source_dest_instruction("MOVE", |src, dest| Instruction::Move { src, dest });
    /// ```
    fn test_source_dest_instruction(
        instruction: &str,
        gen_instruction: impl Fn(AddressMode, AddressMode) -> Instruction,
    ) {
        for (asm, src, dest) in [
            (
                "a0, a1",
                AddressMode::RegisterDirect {
                    register: Address(AddressRegister::A0),
                },
                AddressMode::RegisterDirect {
                    register: Address(AddressRegister::A1),
                },
            ),
            (
                "(12, a5), d3",
                AddressMode::RegisterIndirectWithDisplacement {
                    register: AddressRegister::A5,
                    displacement: 12,
                },
                AddressMode::RegisterDirect {
                    register: Data(DataRegister::D3),
                },
            ),
        ] {
            let mut interpreter = AssemblyInterpreter::new();
            assert_eq!(
                interpreter
                    .parse(format!("{} {}", instruction, asm))
                    .unwrap()
                    .0,
                gen_instruction(src, dest)
            );
        }
    }

    #[test]
    fn parse_move() {
        test_source_dest_instruction("MOVE", |src, dest| Instruction::Move { src, dest });
    }

    #[test]
    fn parse_add() {
        test_source_dest_instruction("ADD", |src, dest| Instruction::Add { src, dest });
    }

    #[test]
    fn parse_subtract() {
        test_source_dest_instruction("SUB", |src, dest| Instruction::Subtract { src, dest });
    }

    #[test]
    fn parse_unsigned_multiplication() {
        test_source_dest_instruction("MULU", |src, dest| Instruction::MultiplyUnsigned {
            src,
            dest,
        });
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
                (AddressMode::RegisterDirect { register }, None)
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
                (AddressMode::RegisterIndirect { register }, None)
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
                (
                    AddressMode::RegisterIndirectPostIncrement { register },
                    None
                )
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
                (AddressMode::RegisterIndirectPreDecrement { register }, None)
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
                (
                    AddressMode::RegisterIndirectWithDisplacement {
                        register,
                        displacement
                    },
                    None
                )
            );
        }
    }

    #[test]
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
                (
                    AddressMode::RegisterIndirectIndexed {
                        address_register,
                        index_register,
                        displacement,
                    },
                    Some(size)
                )
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
                (
                    AddressMode::MemoryPostIndexed {
                        address_register,
                        index_register,
                        base_displacement,
                        outer_displacement,
                    },
                    Some(size)
                )
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
                (
                    AddressMode::MemoryPreIndexed {
                        address_register,
                        index_register,
                        base_displacement,
                        outer_displacement,
                    },
                    Some(size)
                )
            );
        }
    }

    #[test]
    fn parse_to_operand_pc_indirect_with_displacement() {
        for (operand, displacement) in [("(1, pc)", 1), ("(8, pc)", 8), ("(952, pc)", 952)] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                (
                    AddressMode::ProgramCounterIndirectWithDisplacement { displacement },
                    None
                )
            );
        }
    }

    #[test]
    fn parse_to_operand_pc_indirect_indexed() {
        for (operand, displacement, index_register, size) in [
            ("(1, pc, d3.b)", 1, Data(DataRegister::D3), Byte),
            ("(8, pc, a4.w)", 8, Address(AddressRegister::A4), Word),
            ("(952, pc, d5.l)", 952, Data(DataRegister::D5), Long),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                (
                    AddressMode::ProgramCounterIndirectIndexed {
                        index_register,
                        displacement,
                    },
                    Some(size)
                )
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
                (
                    AddressMode::ProgramCounterMemoryIndirectPostIndexed {
                        index_register,
                        base_displacement,
                        outer_displacement,
                    },
                    Some(size)
                )
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
                (
                    AddressMode::ProgramCounterMemoryIndirectPreIndexed {
                        index_register,
                        base_displacement,
                        outer_displacement,
                    },
                    Some(size)
                )
            );
        }
    }

    #[test]
    fn parse_to_operand_absolute() {
        for (operand, address, size) in [("($400).w", 0x400, Word), ("($b4a).l", 0xB4A, Long)] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                (AddressMode::Absolute { address }, Some(size))
            );
        }
    }

    #[test]
    fn parse_to_operand_immediate() {
        // TODO: should this be a byte, word, or long?
        for (operand, value) in [("#$400", 0x400)] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand, &DUMMY_INSTRUCTION).unwrap(),
                (AddressMode::Immediate { value }, None)
            );
        }
    }
}
