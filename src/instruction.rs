// Rust doesn't support these types so we use the next smallest representation
// for these types instead.
pub type U4 = u8;
pub type U12 = u16;

/// A CHIP-8 instruction.
#[derive(Debug, PartialEq, Eq)]
pub struct Instruction {
    opcode: U4,
    x: U4,
    y: U4,
    n: U4,
    nn: u8,
    nnn: U12,
}

impl Instruction {
    /// Parses an [`Instruction`] from two bytes.
    pub fn parse(first: u8, second: u8) -> Self {
        Self {
            opcode: (first >> 4) & 0xF,
            x: first & 0xF,
            y: (second >> 4) & 0xF,
            n: second & 0xF,
            nn: second,
            nnn: u16::from(first & 0xF) << 8 | u16::from(second),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        assert_eq!(
            Instruction::parse(0b11110000, 0b11110000),
            Instruction {
                opcode: 0b1111,
                x: 0b0000,
                y: 0b1111,
                n: 0b0000,
                nn: 0b11110000,
                nnn: 0b0000000011110000,
            }
        );
    }
}
