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
        let (src, dest, size_override) = AddressMode::from_m68kdecode(
            decoded.instruction.operands[0].clone(),
            decoded.instruction.operands[1].clone(),
        )
        .unwrap();

        let size = if decoded.instruction.size == 0 {
            size_override.unwrap_or_else(|| {
                eprintln!(
                    "Warning: no size override and instruction size is 0. Defaulting to Long."
                );
                OperandSize::Long
            })
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
            Operation::ADD | Operation::ADDI | Operation::ADDA | Operation::ADDQ => {
                Instruction::Add {
                    src: src.unwrap(),
                    dest: dest.unwrap(),
                }
            }
            Operation::SUB | Operation::SUBI | Operation::SUBA => Instruction::Subtract {
                src: src.unwrap(),
                dest: dest.unwrap(),
            },
            Operation::MULU => Instruction::MultiplyUnsigned {
                src: src.unwrap(),
                dest: dest.unwrap(),
            },
            // TODO: should movea alter the address mode to be indirect?
            // TODO: support reading from multiple registers to a pre/postdecrement register
            // Necessary for things like `movem %a5/%a6, (%sp)-` which is used in Macintosh ROM calling conventions
            Operation::MOVE | Operation::MOVEA | Operation::MOVEM => Instruction::Move {
                src: src.unwrap(),
                dest: dest.unwrap(),
            },
            Operation::EOR | Operation::EORI => Instruction::ExclusiveOr {
                src: src.unwrap(),
                dest: dest.unwrap(),
            },
            Operation::OR | Operation::ORI => Instruction::InclusiveOr {
                src: src.unwrap(),
                dest: dest.unwrap(),
            },
            Operation::AND | Operation::ANDI => Instruction::And {
                src: src.unwrap(),
                dest: dest.unwrap(),
            },
            // TODO: figure out what ROL means and how it is different from ROXL
            Operation::ROXL | Operation::ROL => Instruction::RotateLeft {
                to_rotate: dest.unwrap(),
                rotate_amount: src.unwrap(),
            },
            Operation::JMP => Instruction::JumpTo {
                address: src.unwrap(),
            },
            Operation::CHK => Instruction::BoundsCheck {
                value: dest.unwrap(),
                bound: src.unwrap(),
            },
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
