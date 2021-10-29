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

use crate::{cpu::{CPU, CPUError, addressing::AddressMode}, ram::Memory};

use super::Instruction;

// Can switch to an enum if perf is an issue
struct Move {
    source: AddressMode,
    destination: AddressMode,
}

impl Instruction for Move {
    fn execute(&self, cpu: &mut CPU<impl Memory>) -> Result<(), CPUError> {
        unimplemented!();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{OperandSize, cpu::{CPU, addressing::AddressMode}, ram::VecBackedMemory};

    static ADDRESS: u32 = 0x40;
    static VALUE: u32 = 0xDEADBEEF;

    #[test]
    fn move_instruction() {
        let mut cpu = CPU::<VecBackedMemory>::new(1024);
        let destination = AddressMode::Absolute { address: ADDRESS, size: OperandSize::Long };
        let source = AddressMode::Immediate { value: VALUE, size: OperandSize::Long };
        let instruction = Move { source, destination };

        instruction.execute(&mut cpu).unwrap();
        assert_eq!(cpu.memory.read_long(ADDRESS).unwrap(), VALUE);
    }
}