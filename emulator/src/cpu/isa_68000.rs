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
    AddBCD {
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
    BoundsCheck {
        bound: AddressMode,
        value: AddressMode,
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
                let val = src
                    .get_value(cpu, size)?
                    .wrapping_add(dest.get_value(cpu, size)?);
                dest.set_value(cpu, val)
            }
            Instruction::Subtract { src, dest } => {
                let val = src
                    .get_value(cpu, size)?
                    .wrapping_sub(dest.get_value(cpu, size)?);
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
            // Could use a cleaner API like `src.modify(cpu, size, |val| val.and(dest.get_value(cpu, size)?))`
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
                let val = address.get_value(cpu, OperandSize::Long)?;
                let int: u32 = val.into();
                eprintln!("Jumping to {:X} (current PC value: {:?})", int, cpu.registers.get(Register::ProgramCounter));
                cpu.registers.set(Register::ProgramCounter, val);
                Ok(())
            }
            Instruction::BoundsCheck { bound, value } => {
                let val: u32 = value.get_value(cpu, size)?.into();
                let val = val as i32;
                let bound: u32 = bound.get_value(cpu, size)?.into();

                if val > bound as i32 || val < 0 {
                    todo!("exception handling")
                } else {
                    Ok(())
                }
            }
            Instruction::NoOp => Ok(()),

            _ => unimplemented!("instruction {:?}", self),
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

    /// Genernates a unit test with the given test cases for an instruction that uses the src/dest format
    macro_rules! test_instruction {
        ($( #[$meta:meta], )? $function_name:ident, $variant:ident, $op1:ident, $op2:ident, $size:ident, $( ($src:expr, $dest:expr) => $result:expr ),*) => {
            #[test]
            $( #[$meta] )?
            fn $function_name() {
                for (a, b, result) in [$( ($src, $dest, $result) ),*].iter() {
                    let cpu = &mut CPU::<VecBackedMemory>::new(1024);

                    let src = AddressMode::Immediate { value: *a };
                    let dest = AddressMode::Absolute { address: ADDRESS };
                    dest.set_value(cpu, M68kInteger::$size(*b)).unwrap();

                    let instruction = Instruction::$variant {
                        $op1: src,
                        $op2: dest.clone(),
                    };

                    instruction.execute(cpu, OperandSize::$size).unwrap();
                    assert_eq!(
                        dest.get_value(cpu, OperandSize::$size).unwrap(),
                        M68kInteger::$size(*result)
                    );
                }
            }
        };

        // Default to src/dest/long
        ($( #[$meta:meta], )? $function_name:ident, $variant:ident, $( ($src:expr, $dest:expr) => $result:expr ),*) => {
            test_instruction! { $( #[$meta], )? $function_name, $variant, src, dest, Long, $( ($src, $dest) => $result ),* }
        };
    }

    // TODO: figure out about setting the status register
    test_instruction!(add, Add, (1, 2) => 3, (0, 0) => 0, (0xFFFFFFFF, 1) => 0x00000000);
    test_instruction!(subtract, Subtract, (1, 2) => 0xFFFFFFFF, (0, 0) => 0, (20, 10) => 10);
    test_instruction!(multiply_unsigned, MultiplyUnsigned, (1, 2) => 2, (0, 0) => 0, (20, 10) => 200, (0x80000000, 2) => 0);
    test_instruction!(xor, ExclusiveOr, (1, 2) => 3, (0, 0) => 0, (7, 3) => 4, (0xAAAA, 0x15555) => 0x1FFFF);
    test_instruction!(or, InclusiveOr, (1, 2) => 3, (0, 0) => 0, (7, 3) => 7);
    test_instruction!(and, And, (2, 4) => 0, (0, 0) => 0, (0xCD, 0xAB) => 0x89);
    test_instruction!(rotate_left, RotateLeft, rotate_amount, to_rotate, Byte, (2, 0b10101011) => 0b10101110, (0, 0) => 0, (2, 0b11101011) => 0b10101111);
    // I don't really know how BCD works, and it seems like there are multiple ways to represent numbers :S
    // Let's hope a real 68k uses the same representation as is documented on Wikipedia!
    // https://en.wikipedia.org/wiki/Binary-coded_decimal#Background
    // The Macintosh ROMs seem to use BCD instructions, too, so this actually will need to be correct eventually.
    // Also, TODO: understand the "extend bit" mentioned at http://wpage.unina.it/rcanonic/didattica/ce1/docs/68000.pdf
    test_instruction!(
        #[ignore],
        abcd, AddBCD,
        // 2 + 2 = 4
        (0b0010, 0b0010) => 0b0100,
        // 13 + 21 = 34
        (0b0011_0001, 0b0001_0010) => 0b0011_0100,
        // 13 + 9 = 22
        (0b0011_0001, 0b1001) => 0b0010_0010
    );

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
    fn move_instruction() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let dest = AddressMode::Absolute { address: ADDRESS };
        let src = AddressMode::Immediate { value: VALUE };
        let instruction = Instruction::Move { src, dest };

        instruction.execute(&mut cpu, OperandSize::Long).unwrap();
        assert_eq!(cpu.memory.read_long(ADDRESS).unwrap(), VALUE);
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
