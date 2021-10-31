//! Emulation of the 68000's registers
//!
//! There are 7 data registers, 7 address registers, a program counter, and the status register.

use std::fmt::Display;

use crate::{hex_format_long, hex_format_word};

pub type RegisterValue = u32;

/// The status register is smaller and has its own methods
#[derive(Debug, Clone, Copy, PartialEq, Eq)] // remove if perf issue
pub enum Register {
    Data(DataRegister),
    Address(AddressRegister),
    ProgramCounter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)] // remove if perf issue
pub enum DataRegister {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)] // remove if perf issue
pub enum AddressRegister {
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    /// Is the stack pointer
    A7,
}

#[derive(Default)]
pub struct Registers {
    // Data registers
    d0: RegisterValue,
    d1: RegisterValue,
    d2: RegisterValue,
    d3: RegisterValue,
    d4: RegisterValue,
    d5: RegisterValue,
    d6: RegisterValue,
    d7: RegisterValue,

    // Address registers
    a0: RegisterValue,
    a1: RegisterValue,
    a2: RegisterValue,
    a3: RegisterValue,
    a4: RegisterValue,
    a5: RegisterValue,
    a6: RegisterValue,
    a7: RegisterValue,

    /// Program counter
    pc: RegisterValue,

    /// Status register (top 8 bytes = system byte, bottom 8 bytes = CCR)
    status: u16,
}

impl Registers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, register: impl Into<Register>) -> RegisterValue {
        match register.into() {
            Register::Data(reg) => self.get_data_register(reg),
            Register::Address(reg) => self.get_address_register(reg),
            Register::ProgramCounter => self.pc,
        }
    }

    pub fn get_data_register(&self, register: DataRegister) -> RegisterValue {
        match register {
            DataRegister::D0 => self.d0,
            DataRegister::D1 => self.d1,
            DataRegister::D2 => self.d2,
            DataRegister::D3 => self.d3,
            DataRegister::D4 => self.d4,
            DataRegister::D5 => self.d5,
            DataRegister::D6 => self.d6,
            DataRegister::D7 => self.d7,
        }
    }

    pub fn get_address_register(&self, register: AddressRegister) -> RegisterValue {
        match register {
            AddressRegister::A0 => self.a0,
            AddressRegister::A1 => self.a1,
            AddressRegister::A2 => self.a2,
            AddressRegister::A3 => self.a3,
            AddressRegister::A4 => self.a4,
            AddressRegister::A5 => self.a5,
            AddressRegister::A6 => self.a6,
            AddressRegister::A7 => self.a7,
        }
    }

    pub fn set(&mut self, register: Register, new_value: impl Into<RegisterValue> + Copy) {
        let new_value = new_value.into();
        match register {
            Register::Data(reg) => self.set_data_register(reg, new_value),
            Register::Address(reg) => self.set_address_register(reg, new_value),
            Register::ProgramCounter => self.pc = new_value,
        }
    }

    pub fn set_address_register(
        &mut self,
        register: AddressRegister,
        new_value: impl Into<RegisterValue> + Copy,
    ) {
        let new_value = new_value.into();
        match register {
            AddressRegister::A0 => self.a0 = new_value,
            AddressRegister::A1 => self.a1 = new_value,
            AddressRegister::A2 => self.a2 = new_value,
            AddressRegister::A3 => self.a3 = new_value,
            AddressRegister::A4 => self.a4 = new_value,
            AddressRegister::A5 => self.a5 = new_value,
            AddressRegister::A6 => self.a6 = new_value,
            AddressRegister::A7 => self.a7 = new_value,
        }
    }

    pub fn set_data_register(
        &mut self,
        register: DataRegister,
        new_value: impl Into<RegisterValue> + Copy,
    ) {
        let new_value = new_value.into();
        match register {
            DataRegister::D0 => self.d0 = new_value,
            DataRegister::D1 => self.d1 = new_value,
            DataRegister::D2 => self.d2 = new_value,
            DataRegister::D3 => self.d3 = new_value,
            DataRegister::D4 => self.d4 = new_value,
            DataRegister::D5 => self.d5 = new_value,
            DataRegister::D6 => self.d6 = new_value,
            DataRegister::D7 => self.d7 = new_value,
        }
    }

    pub fn get_status_register(&self) -> u16 {
        self.status
    }

    pub fn set_status_register(&mut self, new_value: u16) {
        self.status = new_value;
    }
}

impl Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "A0: 0x{} | A1: 0x{} | A2: 0x{} | A3: 0x{} | A4: 0x{} | A5: 0x{} | A6: 0x{} | A7: 0x{}",
            hex_format_long(self.a0), hex_format_long(self.a1), hex_format_long(self.a2),
            hex_format_long(self.a3), hex_format_long(self.a4), hex_format_long(self.a5),
            hex_format_long(self.a6), hex_format_long(self.a7)
        )?;
        writeln!(
            f,
            "D0: 0x{} | D1: 0x{} | D2: 0x{} | D3: 0x{} | D4: 0x{} | D5: 0x{} | D6: 0x{} | D7: 0x{}",
            hex_format_long(self.d0), hex_format_long(self.d1), hex_format_long(self.d2),
            hex_format_long(self.d3), hex_format_long(self.d4), hex_format_long(self.d5),
            hex_format_long(self.d6), hex_format_long(self.d7)
        )?;
        writeln!(f, "Program Counter: 0x{} | Status: 0x{}", hex_format_long(self.pc), hex_format_word(self.status))
    }
}
