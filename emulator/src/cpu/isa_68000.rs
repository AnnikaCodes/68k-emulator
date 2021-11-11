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
    cpu::{addressing::AddressMode, registers::Register, CPU},
    ram::Memory,
    EmulationError, OperandSize,
};

#[derive(Debug, PartialEq)]
pub enum Instruction {
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
    JumpTo {
        address: AddressMode,
    },
    NoOp,
}

impl Instruction {
    pub fn execute(
        &self,
        cpu: &mut CPU<impl Memory>,
        size: OperandSize,
    ) -> Result<(), EmulationError> {
        match self {
            Instruction::Add { src, dest } => {
                let val = src.get_value(cpu, size)? + dest.get_value(cpu, size)?;
                dest.set_value(cpu, val)
            }
            Instruction::Subtract { src, dest } => {
                let val = src.get_value(cpu, size)? - dest.get_value(cpu, size)?;
                dest.set_value(cpu, val)
            }
            Instruction::MultiplyUnsigned { src, dest } => {
                let val = src
                    .get_value(cpu, size)?
                    .wrapping_mul(dest.get_value(cpu, size)?);
                dest.set_value(cpu, val)
            }
            Instruction::Move { src, dest } => {
                let val = src.get_value(cpu, size)?;
                dest.set_value(cpu, val)
            }
            Instruction::ExclusiveOr { src, dest } => {
                let val = src.get_value(cpu, size)?.xor(dest.get_value(cpu, size)?);
                dest.set_value(cpu, val)
            }
            Instruction::InclusiveOr { src, dest } => {
                let val = src.get_value(cpu, size)?.or(dest.get_value(cpu, size)?);
                dest.set_value(cpu, val)
            }
            Instruction::And { src, dest } => {
                let val = src.get_value(cpu, size)?.and(dest.get_value(cpu, size)?);
                dest.set_value(cpu, val)
            }
            Instruction::RotateLeft {
                to_rotate,
                rotate_amount,
            } => {
                let val = to_rotate
                    .get_value(cpu, size)?
                    .rotate_left(rotate_amount.get_value(cpu, size)?);
                to_rotate.set_value(cpu, val)
            }
            Instruction::JumpTo { address } => {
                let val = address.get_value(cpu, size)?;
                cpu.registers.set(Register::ProgramCounter, val);
                Ok(())
            }
            Instruction::NoOp => Ok(()),
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
        let dest = AddressMode::Absolute { address: ADDRESS };
        let src = AddressMode::Immediate { value: VALUE };
        let instruction = Instruction::Move { src, dest };

        instruction.execute(&mut cpu, OperandSize::Long).unwrap();
        assert_eq!(cpu.memory.read_long(ADDRESS).unwrap(), VALUE);
    }

    #[test]
    fn add_instruction() {
        for (a, b, result) in [(1, 2, 3), (0, 0, 0), (0xFFFFFFFF, 1, 0x00000000u32)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate { value: a };
            let dest = AddressMode::Absolute { address: ADDRESS };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = Instruction::Add {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu, OperandSize::Long).unwrap();
            assert_eq!(
                dest.get_value(cpu, OperandSize::Long).unwrap(),
                M68kInteger::Long(result)
            );
        }
    }

    #[test]
    fn subtract_instruction() {
        // 0x1 - 0x2 = 0xFFFFFFFF because of wrapping... as an i32 it would work.
        for (a, b, result) in [(1, 2, 0xFFFFFFFF), (0, 0, 0), (20, 10, 10)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate { value: a };
            let dest = AddressMode::Absolute { address: ADDRESS };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = Instruction::Subtract {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu, OperandSize::Long).unwrap();
            assert_eq!(
                dest.get_value(cpu, OperandSize::Long).unwrap(),
                M68kInteger::Long(result)
            );
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

            let src = AddressMode::Immediate { value: a };
            let dest = AddressMode::Absolute { address: ADDRESS };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = Instruction::MultiplyUnsigned {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu, OperandSize::Long).unwrap();
            assert_eq!(
                dest.get_value(cpu, OperandSize::Long).unwrap(),
                M68kInteger::Long(result)
            );
        }
    }

    #[test]
    fn exclusive_or() {
        for (a, b, result) in [(1, 2, 3), (0, 0, 0), (0xAAAA, 0x15555, 0x1FFFF)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate { value: a };
            let dest = AddressMode::Absolute { address: ADDRESS };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = Instruction::ExclusiveOr {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu, OperandSize::Long).unwrap();
            assert_eq!(
                dest.get_value(cpu, OperandSize::Long).unwrap(),
                M68kInteger::Long(result)
            );
        }
    }

    #[test]
    fn inclusive_or() {
        for (a, b, result) in [(1, 2, 3), (0, 0, 0), (0xAAAA, 0x15555, 0x1FFFF)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate { value: a };
            let dest = AddressMode::Absolute { address: ADDRESS };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = Instruction::InclusiveOr {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu, OperandSize::Long).unwrap();
            assert_eq!(
                dest.get_value(cpu, OperandSize::Long).unwrap(),
                M68kInteger::Long(result)
            );
        }
    }

    // TODO: use a macro to simplify these tests?
    // eg test_source_dest!(Instruction::And, [(2, 4, 0), (0, 0, 0), (0xCD, 0xAB, 0x89)]);
    #[test]
    fn and() {
        for (a, b, result) in [(2, 4, 0), (0, 0, 0), (0xCD, 0xAB, 0x89)] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let src = AddressMode::Immediate { value: a };
            let dest = AddressMode::Absolute { address: ADDRESS };
            dest.set_value(cpu, M68kInteger::Long(b)).unwrap();

            let instruction = Instruction::And {
                src,
                dest: dest.clone(),
            };

            instruction.execute(cpu, OperandSize::Long).unwrap();
            assert_eq!(
                dest.get_value(cpu, OperandSize::Long).unwrap(),
                M68kInteger::Long(result)
            );
        }
    }

    #[test]
    fn rotate_left() {
        for (a, b, result) in [
            (2, 0b10101011, 0b10101110),
            (0, 0, 0),
            (2, 0b11101011, 0b10101111),
        ] {
            let cpu = &mut CPU::<VecBackedMemory>::new(1024);

            let rotate_amount = AddressMode::Immediate { value: a };
            let to_rotate = AddressMode::Absolute { address: ADDRESS };
            to_rotate.set_value(cpu, M68kInteger::Byte(b)).unwrap();

            let instruction = Instruction::RotateLeft {
                to_rotate: to_rotate.clone(),
                rotate_amount,
            };

            instruction.execute(cpu, OperandSize::Byte).unwrap();
            assert_eq!(
                to_rotate.get_value(cpu, OperandSize::Byte).unwrap(),
                M68kInteger::Byte(result)
            );
        }
    }

    #[test]
    fn jump() {
        let cpu = &mut CPU::<VecBackedMemory>::new(1024);
        let instruction = Instruction::JumpTo {
            address: AddressMode::Immediate { value: ADDRESS },
        };

        assert_ne!(cpu.registers.get(Register::ProgramCounter), ADDRESS);
        instruction.execute(cpu, OperandSize::Long).unwrap();
        assert_eq!(cpu.registers.get(Register::ProgramCounter), ADDRESS);
    }

    #[test]
    fn no_op() {
        let cpu = &mut CPU::<VecBackedMemory>::new(1024);
        let instruction = Instruction::NoOp;

        let initial_state = format!("{}", cpu);
        instruction.execute(cpu, OperandSize::Long).unwrap();
        assert_eq!(format!("{}", cpu), initial_state);
    }
}
