use std::fmt::Display;

use crate::{hex_format_byte, EmulationError, M68kInteger, OperandSize};

// Emulation of Random-Access Memory (RAM)

pub trait Memory: Display {
    fn new(size_in_bytes: usize) -> Self;
    fn read_byte(&self, address: u32) -> Result<u8, EmulationError>;
    fn write_byte(&mut self, address: u32, value: u8) -> Result<(), EmulationError>;

    fn read(&self, address: u32, size: OperandSize) -> Result<M68kInteger, EmulationError> {
        match size {
            OperandSize::Byte => Ok(M68kInteger::Byte(self.read_byte(address)?)),
            OperandSize::Word => Ok(M68kInteger::Word(self.read_word(address)?)),
            OperandSize::Long => Ok(M68kInteger::Long(self.read_long(address)?)),
        }
    }

    fn read_bytes(&self, address: u32, len: u32) -> Result<Vec<u8>, EmulationError> {
        let mut bytes = Vec::with_capacity(len as usize);
        for i in 0..len {
            bytes.push(self.read_byte(address + i)?);
        }
        Ok(bytes)
    }

    fn read_word(&self, address: u32) -> Result<u16, EmulationError> {
        let high_byte = self.read_byte(address)?;
        let low_byte = self.read_byte(address + 1)?;
        Ok(((high_byte as u16) << 8) + low_byte as u16)
    }

    fn read_long(&self, address: u32) -> Result<u32, EmulationError> {
        let high_word = self.read_word(address)?;
        let low_word = self.read_word(address + 2)?;
        Ok(((high_word as u32) << 16) + low_word as u32)
    }

    fn write(&mut self, address: u32, value: M68kInteger) -> Result<(), EmulationError> {
        match value {
            M68kInteger::Byte(value) => self.write_byte(address, value),
            M68kInteger::Word(value) => self.write_word(address, value),
            M68kInteger::Long(value) => self.write_long(address, value),
        }
    }
    fn write_bytes(&mut self, address: u32, value: Vec<u8>) -> Result<(), EmulationError> {
        for (i, byte) in value.iter().enumerate() {
            self.write_byte(address + i as u32, *byte)?;
        }
        Ok(())
    }

    fn write_word(&mut self, address: u32, value: u16) -> Result<(), EmulationError> {
        let low_byte = (value & 0x00FF) as u8;
        let high_byte = (value >> 8) as u8;

        self.write_byte(address, high_byte)?;
        self.write_byte(address + 1, low_byte)
    }

    fn write_long(&mut self, address: u32, long: u32) -> Result<(), EmulationError> {
        let low_word = (long & 0x0000FFFF) as u16;
        let high_word = (long >> 16) as u16;

        self.write_word(address, high_word)?;
        self.write_word(address + 2, low_word)
    }
}

/// Naive Vec<u8> implementation of RAM
pub struct VecBackedMemory {
    random_access_buf: Vec<u8>,
    // TODO: implement memory mapping
}

impl Display for VecBackedMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // remove long stretches of 0s
        let mut showed_last = false;
        let interesting_memory =
            self.random_access_buf
                .iter()
                .enumerate()
                .filter_map(|(idx, byte)| {
                    if self.random_access_buf.get(idx + 1) != Some(&0)
                        || idx == 0
                        || self.random_access_buf.get(idx - 1) != Some(&0)
                    {
                        // Show the first and last zero bytes in a zero-sequence
                        showed_last = true;
                        Some(hex_format_byte(*byte))
                    } else if *byte == 0 && showed_last {
                        // Replace first zero byte with a ...
                        showed_last = false;
                        Some(String::from("..."))
                    } else if *byte == 0
                        && self.random_access_buf.get(idx + 1) == Some(&0)
                        && self.random_access_buf.get(idx - 1) == Some(&0)
                    {
                        // Zero byte
                        showed_last = false;
                        None
                    } else {
                        showed_last = true;
                        Some(hex_format_byte(*byte))
                    }
                });

        write!(
            f,
            "RAM (Vec-backed): {}",
            interesting_memory.collect::<Vec<_>>().join(" ")
        )
    }
}

impl Memory for VecBackedMemory {
    fn new(size_in_bytes: usize) -> Self {
        Self {
            random_access_buf: vec![0; size_in_bytes],
        }
    }

    fn read_byte(&self, address: u32) -> Result<u8, EmulationError> {
        match self.random_access_buf.get(address as usize) {
            Some(byte) => Ok(*byte),
            None => Err(EmulationError::MemoryOutOfBoundsAccess(address)),
        }
    }

    fn write_byte(&mut self, address: u32, value: u8) -> Result<(), EmulationError> {
        match self.random_access_buf.get_mut(address as usize) {
            Some(byte) => {
                *byte = value;
                Ok(())
            }
            None => Err(EmulationError::MemoryOutOfBoundsAccess(address)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SIZE: usize = 0x400; // 1KB
    static ADDRESS: u32 = 0x201;

    #[test]
    fn byte_rw() {
        let byte = 0xAB;
        for mut ram_impl in [VecBackedMemory::new(SIZE)] {
            ram_impl.write_byte(ADDRESS, byte).unwrap();
            assert_eq!(ram_impl.read_byte(ADDRESS).unwrap(), byte);
        }
    }

    #[test]
    fn word_rw() {
        let word = 0xDEAD;
        for mut ram_impl in [VecBackedMemory::new(SIZE)] {
            ram_impl.write_word(ADDRESS, word).unwrap();
            assert_eq!(ram_impl.read_word(ADDRESS).unwrap(), word);
        }
    }

    #[test]
    fn long_rw() {
        let long = 0xDEADBEEF;
        for mut ram_impl in [VecBackedMemory::new(SIZE)] {
            ram_impl.write_long(ADDRESS, long).unwrap();
            assert_eq!(ram_impl.read_long(ADDRESS).unwrap(), long);
        }
    }

    #[test]
    fn multiple_bytes_rw() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78, 0x9A];
        for mut ram_impl in [VecBackedMemory::new(SIZE)] {
            ram_impl.write_bytes(ADDRESS, bytes.clone()).unwrap();
            assert_eq!(
                ram_impl
                    .read_bytes(ADDRESS, bytes.len().try_into().unwrap())
                    .unwrap(),
                bytes
            );

            // bonus!
            assert_eq!(ram_impl.read_long(ADDRESS).unwrap(), 0x12345678);
            assert_eq!(ram_impl.read_byte(ADDRESS + 4).unwrap(), 0x9A);
        }
    }

    #[test]
    fn display_does_not_include_lots_of_zeroes() {
        for ram_impl in [VecBackedMemory::new(SIZE)] {
            let display = format!("{}", ram_impl);
            let zeroes = display.chars().filter(|c| *c == '0').count();
            assert!(zeroes < 10, "too many zeroes not filtered out: {}", zeroes);
        }
    }
}
