//! CPU instructions live here!
//!
//! Each CPU has its own file, so you can do things like
//! ```
//! use emulator::cpu::instructions::isa_68000;
//! ```
//! to only use the 68000 instructions.
//!
//! However, we don't support non-68000s yet, so it's not terribly relevant.

use crate::ram::Memory;

use super::{CPU, CPUError};
pub mod isa_68000;

/// A CPU instruction
pub trait Instruction {
    fn execute(&self, cpu: &mut CPU<impl Memory>) -> Result<(), CPUError>;
}

