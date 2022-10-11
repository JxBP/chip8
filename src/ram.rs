use anyhow::{bail, Result};

pub const RAM_SIZE: usize = 4096;

/// Ram is a safe wrapper to access an array serving as memory for the emulator.
/// Addresses are checked for validity to prevent panics when indexing the array
/// out of bounds.
pub struct Ram([u8; RAM_SIZE]);

impl Ram {
    /// Updates the value at an address.
    ///
    /// # Errors
    /// An error might occur when the address is not in the bounds of the
    /// memory.
    pub fn set(&mut self, address: usize, value: u8) -> Result<()> {
        is_valid_address(address)?;
        self.0[address] = value;
        Ok(())
    }

    /// Gets the value at an address.
    ///
    /// # Errors
    /// An error might occur when the address is not in the bounds of the
    /// memory.
    pub fn get(&self, address: usize) -> Result<u8> {
        is_valid_address(address)?;
        Ok(self.0[address])
    }

    /// Might be removed soon.
    pub fn get_slice(&self, address: usize, length: usize) -> Result<&[u8]> {
        is_valid_address(address)?;
        is_valid_address(address + length)?;
        Ok(&self.0[address..(address + length)])
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self([0u8; RAM_SIZE])
    }
}

/// Checks if an address would panic if accessed.
fn is_valid_address(address: usize) -> Result<()> {
    if address >= RAM_SIZE {
        bail!("Address out of bounds: {} >= {}", address, RAM_SIZE);
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_addresses() {
        assert!(is_valid_address(RAM_SIZE).is_err());
        assert!(is_valid_address(RAM_SIZE + 10).is_err());
    }

    #[test]
    fn test_valid_addresses() {
        assert!(is_valid_address(0).is_ok());
        assert!(is_valid_address(RAM_SIZE - 1).is_ok());
    }

    #[test]
    fn test_get_valid_slice() {
        let ram = Ram::default();
        assert!(ram.get_slice(0, RAM_SIZE - 1).is_ok());
        assert!(ram.get_slice(RAM_SIZE / 4, RAM_SIZE / 2).is_ok());
    }

    #[test]
    fn test_get_invalid_slice() {
        let ram = Ram::default();
        assert!(ram.get_slice(0, RAM_SIZE).is_err());
        assert!(ram.get_slice(RAM_SIZE / 2, RAM_SIZE + 5).is_err());
    }
}
