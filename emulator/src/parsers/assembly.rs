//! Parses assembly code

use crate::cpu::{addressing::AddressMode, isa_68000::*};

use super::{Interpreter, ParseError};

pub struct AssemblyInterpreter {}

impl AssemblyInterpreter {
    pub fn new() -> Self {
        AssemblyInterpreter {}
    }

    /// Parses an operand to an address
    ///
    /// TODO: figure out how different operand sizes are represented & handle accordingly in unit tests
    fn parse_to_operand(op_string: &str) -> Result<AddressMode, ParseError> {
        unimplemented!();
    }
}

impl Interpreter<String> for AssemblyInterpreter {
    fn parse_instruction(&mut self, source: String) -> Result<InstructionFor68000, ParseError> {
        let lowercase_source = source.to_lowercase();
        let (instruction_token, rest) = match lowercase_source.trim().split_once(' ') {
            Some(s) => s,
            None => return Err(ParseError::NoInstruction(source)),
        };

        match instruction_token {
            "add" => unimplemented!(),
            "sub" => unimplemented!(),
            "move" => {
                let (source_token, destination_token) = rest
                    // TODO: this doesnt actually work, as addressing can include commas....
                    // We probably need to iterate over `rest` and ignore stuff inside parens until we find a comma,
                    // then split at the index.
                    .split_once(',')
                    .ok_or(ParseError::MissingOperand(source))?;
                let source_addr = AssemblyInterpreter::parse_to_operand(source_token)?;
                let destination_addr = AssemblyInterpreter::parse_to_operand(destination_token)?;

                Ok(InstructionFor68000::Move(Move {
                    source: source_addr,
                    destination: destination_addr,
                }))
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

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_move() {
        for (raw, parsed) in [(
            "MOVE a0, a1",
            InstructionFor68000::Move(Move {
                source: AddressMode::RegisterDirect {
                    register: Address(AddressRegister::A0),
                    size: Byte,
                },
                destination: AddressMode::RegisterDirect {
                    register: Address(AddressRegister::A1),
                    size: Byte,
                },
            }),
        )] {
            let mut interpreter = AssemblyInterpreter::new();
            assert_eq!(
                interpreter.parse_instruction(raw.to_string()).unwrap(),
                parsed
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_register_direct() {
        for (operand, register) in [
            ("d0", Data(DataRegister::D0)),
            ("a6", Address(AddressRegister::A6)),
            ("a7", Address(AddressRegister::A7)),
            ("sp", Address(AddressRegister::A7)),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
                AddressMode::RegisterDirect {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_register_indirect() {
        for (operand, register) in [
            ("(a3)", AddressRegister::A6),
            ("(a7)", AddressRegister::A7),
            ("(sp)", AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
                AddressMode::RegisterIndirect {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_register_indirect_postincrement() {
        for (operand, register) in [
            ("(a1)+", AddressRegister::A6),
            ("(a7)+", AddressRegister::A7),
            ("(sp)+", AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
                AddressMode::RegisterIndirectPostIncrement {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_register_indirect_predecrement() {
        for (operand, register) in [
            ("-(a1)", AddressRegister::A6),
            ("-(a7)", AddressRegister::A7),
            ("-(sp)", AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
                AddressMode::RegisterIndirectPreDecrement {
                    register,
                    size: Long,
                }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_register_indirect_displacement() {
        for (operand, displacement, register) in [
            ("(1, a1)", 1, AddressRegister::A6),
            ("(8, a7)", 8, AddressRegister::A7),
            ("(952, sp)", 952, AddressRegister::A7),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
    #[ignore = "not yet implemented"]
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
                "([1,a1] d3.b, 2)", // TODO: does this need + signs
                1,
                2,
                AddressRegister::A1,
                Data(DataRegister::D3),
                Byte,
            ),
            (
                "([8,a7] a4.w, 952)",
                8,
                952,
                AddressRegister::A7,
                Address(AddressRegister::A4),
                Word,
            ),
            (
                "([952,sp] d5.l, 1)",
                952,
                1,
                AddressRegister::A7,
                Data(DataRegister::D5),
                Long,
            ),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
    #[ignore = "not yet implemented"]
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
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
    #[ignore = "not yet implemented"]
    fn parse_to_operand_pc_indirect_indexed() {
        for (operand, displacement, index_register, size) in [
            ("(1, pc, d3.b)", 1, Data(DataRegister::D3), Byte),
            ("(8, pc, a4.w)", 8, Address(AddressRegister::A4), Word),
            ("(952, pc, d5.l)", 952, Data(DataRegister::D5), Long),
        ] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
    #[ignore = "not yet implemented"]
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
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
    #[ignore = "not yet implemented"]
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
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
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
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
                AddressMode::Absolute { address, size }
            );
        }
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn parse_to_operand_immediate() {
        // TODO: should this be a byte, word, or long?
        for (operand, address, size) in [("#$400", 0x400, Word)] {
            assert_eq!(
                AssemblyInterpreter::parse_to_operand(operand).unwrap(),
                AddressMode::Absolute { address, size }
            );
        }
    }
}
