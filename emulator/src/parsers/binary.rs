//! Parses binary machine code

use super::{ParseError, Parser};
use crate::{
    cpu::{addressing::AddressMode, isa_68000::Instruction},
    EmulationError, OperandSize,
};
use colored::Colorize;

use m68kdecode::Operation;
#[derive(Default)]
pub struct MachineCodeParser;

impl Parser<Vec<u8>> for MachineCodeParser {
    fn parse(&mut self, source: Vec<u8>) -> Result<(Instruction, OperandSize, u32), ParseError> {
        let decoded = m68kdecode::decode_instruction(source.as_slice())?;
        dbg!(&decoded);
        let (src, dest, size_override) = AddressMode::from_m68kdecode(
            decoded.instruction.operands[0].clone(),
            decoded.instruction.operands[1].clone(),
        )
        .unwrap();

        let size = if decoded.instruction.size == 0 {
            size_override.expect("Should have a size override when instruction size is 0")
        } else {
            match OperandSize::from_size_in_bytes(decoded.instruction.size) {
                Ok(s) => s,
                Err(e) => match e {
                    EmulationError::InvalidOperandSize(s) => {
                        return Err(ParseError::InvalidOperandSize(s))
                    }
                    _ => panic!("Unexpected error while parsing operand size: {:?}", e),
                },
            }
        };

        let parsed = match decoded.instruction.operation {
            Operation::ADD | Operation::ADDI | Operation::ADDA | Operation::ADDQ => Instruction::Add {
                src,
                dest: dest.unwrap(),
            },
            Operation::SUB | Operation::SUBI | Operation::SUBA => Instruction::Subtract {
                src,
                dest: dest.unwrap(),
            },
            Operation::MULU => Instruction::MultiplyUnsigned {
                src,
                dest: dest.unwrap(),
            },
            // TODO: should movea alter the address mode to be indirect?
            // TODO: implement movem
            Operation::MOVE | Operation::MOVEA | Operation::MOVEM => Instruction::Move {
                src,
                dest: dest.unwrap(),
            },
            Operation::EOR | Operation::EORI => Instruction::ExclusiveOr {
                src,
                dest: dest.unwrap(),
            },
            Operation::OR | Operation::ORI => Instruction::InclusiveOr {
                src,
                dest: dest.unwrap(),
            },
            Operation::AND | Operation::ANDI => Instruction::And {
                src,
                dest: dest.unwrap(),
            },
            // TODO: figure out what ROL means and how it is different from ROXL
            Operation::ROXL | Operation::ROL => Instruction::RotateLeft {
                to_rotate: dest.unwrap(),
                rotate_amount: src,
            },
            Operation::JMP => Instruction::JumpTo { address: src },
            Operation::CHK => Instruction::BoundsCheck { value: dest.unwrap(), bound: src },
            Operation::NOP => Instruction::NoOp,
            _ => {
                eprintln!(
                    "{}: {} for instruction {:?}",
                    "Unknown operation".red().bold(),
                    format!("{:?}", decoded.instruction.operation).cyan().bold(),
                    decoded
                );
                Instruction::NoOp
            }
        };

        Ok((parsed, size, decoded.bytes_used))
    }
}
