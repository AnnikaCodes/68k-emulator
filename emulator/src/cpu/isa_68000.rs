//! Instructions supported by the original Motorola 68000 go here!
//!
//! Refer to https://www.nxp.com/docs/en/reference-manual/M68000PRM.pdf for details on what they do.
//!
//! List of instructions supported:
//! None
//!
//! List of instructions not supported:
//! ABCD (Add Decimal with Extend),
//! ADD (Add),
//! ADDA (Add Address),
//! ADDI (Add Immediate),
//! ADDQ (Add Quick),
//! ADDX (Add with Extend),
//! AND (Logical AND),
//! ANDI (Logical AND Immediate),
//! ANDI (to CCR AND Immediate to Condition Code Register),
//! ANDI (to SR AND Immediate to Status Register),
//! ASL, ASR (Arithmetic Shift Left and Right)
//! Bcc (Branch Conditionally),
//! BCHG (Test Bit and Change),
//! BCLR (Test Bit and Clear),
//! BRA (Branch),
//! BSET (Test Bit and Set),
//! BSR (Branch to Subroutine),
//! BTST (Test Bit),
//! CHK (Check Register Against Bound),
//! CLR (Clear),
//! CMP (Compare),
//! CMPA (Compare Address),
//! CMPI (Compare Immediate),
//! CMPM (Compare Memory to Memory),
//! DBcc (Test Condition, Decrement, and Branch),
//! DIVS (Signed Divide),
//! DIVU (Unsigned Divide),
//! EOR (Logical Exclusive-OR),
//! EORI (Logical Exclusive-OR Immediate),
//! EORI (to CCR Exclusive-OR Immediate to Condition Code Register),
//! EORI (to SR Exclusive-OR Immediate to Status Register),
//! EXG (Exchange Registers),
//! EXT (Sign Extend),
//! ILLEGAL (Take Illegal Instruction Trap),
//! JMP (Jump),
//! JSR (Jump to Subroutine),
//! LEA (Load Effective Address),
//! LINK (Link and Allocate),
//! LSL (LSR Logical Shift Left and Right),
//! MOVE (Move),
//! MOVEA (Move Address),
//! MOVE (to CCR Move to Condition Code Register),
//! MOVE (from SR Move from Status Register),
//! MOVE (to SR Move to Status Register),
//! MOVE (USP Move User Stack Pointer),
//! MOVEM (Move Multiple Registers),
//! MOVEP (Move Peripheral),
//! MOVEQ (Move Quick),
//! MULS (Signed Multiply),
//! MULU (Unsigned Multiply),
//! NBCD (Negate Decimal with Extend),
//! NEG (Negate),
//! NEGX (Negate with Extend),
//! NOP (No Operation),
//! NOT (Logical Complement),
//! OR (Logical Inclusive-OR),
//! ORI (Logical Inclusive-OR Immediate),
//! ORI (to CCR Inclusive-OR Immediate to Condition Code Register),
//! ORI (to SR Inclusive-OR Immediate to Status Register),
//! PEA (Push Effective Address),
//! RESET (Reset External Devices),
//! ROL (ROR Rotate Left and Right),
//! ROXL (ROXR Rotate with Extend Left and Right),
//! RTE (Return from Exception),
//! RTR (Return and Restore),
//! RTS (Return from Subroutine),
//! SBCD (Subtract Decimal with Extend),
//! Scc (Set Conditionally),
//! STOP (Stop),
//! SUB (Subtract),
//! SUBA (Subtract Address),
//! SUBI (Subtract Immediate),
//! SUBQ (Subtract Quick),
//! SUBX (Subtract with Extend),
//! SWAP (Swap Register Words),
//! TAS (Test Operand and Set),
//! TRAP (Trap),
//! TRAPV (Trap on Overflow),
//! TST (Test Operand),
//! UNLK (Unlink)

use crate::{
    cpu::{addressing::AddressMode, CPUError, CPU, registers::Register},
    ram::Memory,
};
use colored::*;
use m68kdecode::{Instruction, Operation};

use super::InstructionSet;

#[derive(Debug, PartialEq)]
pub enum ISA68000 {
    Add {
        src: AddressMode,
        dest: AddressMode,
    },
    Subtract {
        src: AddressMode,
        dest: AddressMode,
    },
    ExclusiveOr {
        src: AddressMode,
        dest: AddressMode,
    },
    InclusiveOr {
        src: AddressMode,
        dest: AddressMode,
    },
    And {
        src: AddressMode,
        dest: AddressMode,
    },
    Move {
        src: AddressMode,
        dest: AddressMode,
    },
    MultiplyUnsigned {
        src: AddressMode,
        dest: AddressMode,
    },
    RotateLeft {
        to_rotate: AddressMode,
        rotate_amount: AddressMode,
    },
    JumpTo { address: AddressMode },
    NoOp,
}

impl InstructionSet for ISA68000 {
    fn execute(&self, cpu: &mut CPU<impl Memory>) -> Result<(), CPUError> {
        match self {
            ISA68000::Add { src, dest } => {
                let val = src.get_value(cpu)? + dest.get_value(cpu)?;
                dest.set_value(cpu, val)
            }
            ISA68000::Subtract { src, dest } => {
                let val = src.get_value(cpu)? - dest.get_value(cpu)?;
                dest.set_value(cpu, val)
            }
            ISA68000::MultiplyUnsigned { src, dest } => {
                let val = src.get_value(cpu)?.wrapping_mul(dest.get_value(cpu)?);
                dest.set_value(cpu, val)
            }
            ISA68000::Move { src, dest } => {
                let val = src.get_value(cpu)?;
                dest.set_value(cpu, val)
            }
            ISA68000::ExclusiveOr { src, dest } => {
                let val = src.get_value(cpu)?.xor(dest.get_value(cpu)?);
                dest.set_value(cpu, val)
            }
            ISA68000::InclusiveOr { src, dest } => {
                let val = src.get_value(cpu)?.or(dest.get_value(cpu)?);
                dest.set_value(cpu, val)
            }
            ISA68000::And { src, dest } => {
                let val = src.get_value(cpu)?.and(dest.get_value(cpu)?);
                dest.set_value(cpu, val)
            }
            ISA68000::RotateLeft {
                to_rotate,
                rotate_amount,
            } => {
                let val = to_rotate
                    .get_value(cpu)?
                    .rotate_left(rotate_amount.get_value(cpu)?);
                to_rotate.set_value(cpu, val)
            }
            ISA68000::JumpTo { address } => {
                let val = address.get_value(cpu)?;
                cpu.registers.set(Register::ProgramCounter, val);
                Ok(())
            }
            ISA68000::NoOp => Ok(()),
        }
    }
}

impl From<Instruction> for ISA68000 {
    fn from(instruction: Instruction) -> Self {
        let (src, dest) = AddressMode::from_m68kdecode(
            instruction.size,
            instruction.operands[0].clone(),
            instruction.operands[1].clone(),
        )
        .unwrap();
        match instruction.operation {
            Operation::ADD | Operation::ADDI | Operation::ADDA => ISA68000::Add {
                src,
                dest: dest.unwrap(),
            },
            Operation::SUB | Operation::SUBI | Operation::SUBA => ISA68000::Subtract {
                src,
                dest: dest.unwrap(),
            },
            Operation::MULU => ISA68000::MultiplyUnsigned {
                src,
                dest: dest.unwrap(),
            },
            Operation::MOVE => ISA68000::Move {
                src,
                dest: dest.unwrap(),
            },
            Operation::EOR | Operation::EORI => ISA68000::ExclusiveOr {
                src,
                dest: dest.unwrap(),
            },
            Operation::OR | Operation::ORI => ISA68000::InclusiveOr {
                src,
                dest: dest.unwrap(),
            },
            Operation::AND | Operation::ANDI => ISA68000::And {
                src,
                dest: dest.unwrap(),
            },
            // TODO: figure out what ROL means and how it is different from ROXL
            Operation::ROXL | Operation::ROL => ISA68000::RotateLeft {
                to_rotate: dest.unwrap(),
                rotate_amount: src,
            },
            Operation::JMP => ISA68000::JumpTo { address: src },
            Operation::NOP => ISA68000::NoOp,
            _ => {
                eprintln!(
                    "{}: {} for instruction {:?}",
                    "Unknown operation".red().bold(), format!("{:?}", instruction.operation).cyan().bold(), instruction
                );
                ISA68000::NoOp
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        cpu::{addressing::AddressMode, CPU},
        ram::VecBackedMemory,
        M68kInteger, OperandSize,
    };

    static ADDRESS: u32 = 0x40;
    static VALUE: u32 = 0xDEADBEEF;

    #[test]
    fn move_instruction() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let dest = AddressMode::Absolute {
            address: ADDRESS,
            size: OperandSize::Long,
        };
        let src = AddressMode::Immediate {
            value: VALUE,
            size: OperandSize::Long,
        };
        let instruction = ISA68000::Move { src, dest };

        instruction.execute(&mut cpu).unwrap();
        assert_eq!(cpu.memory.read_long(ADDRESS).unwrap(), VALUE);
    }

    #[test]
    fn add_instruction() {
        for (a, b, result) in [(1, 2, 3), (0, 0, 0), (0xFFFFFFFF, 1, 0x00000000u32)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate {
                value: a,
                size: OperandSize::Long,
            };
            let dest = AddressMode::Absolute {
                address: ADDRESS,
                size: OperandSize::Long,
            };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = ISA68000::Add {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu).unwrap();
            assert_eq!(dest.get_value(cpu).unwrap(), M68kInteger::Long(result));
        }
    }

    #[test]
    fn subtract_instruction() {
        // 0x1 - 0x2 = 0xFFFFFFFF because of wrapping... as an i32 it would work.
        for (a, b, result) in [(1, 2, 0xFFFFFFFF), (0, 0, 0), (20, 10, 10)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate {
                value: a,
                size: OperandSize::Long,
            };
            let dest = AddressMode::Absolute {
                address: ADDRESS,
                size: OperandSize::Long,
            };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = ISA68000::Subtract {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu).unwrap();
            assert_eq!(dest.get_value(cpu).unwrap(), M68kInteger::Long(result));
        }
    }

    #[test]
    fn multiply_unsigned_instruction() {
        for (a, b, result) in [
            (1u32, 2u32, 2u32),
            (0, 0, 0),
            (20, 10, 200),
            (0x80000000, 2, 0),
        ] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate {
                value: a,
                size: OperandSize::Long,
            };
            let dest = AddressMode::Absolute {
                address: ADDRESS,
                size: OperandSize::Long,
            };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = ISA68000::MultiplyUnsigned {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu).unwrap();
            assert_eq!(dest.get_value(cpu).unwrap(), M68kInteger::Long(result));
        }
    }
}
