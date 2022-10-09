#[derive(Default)]
pub struct Timer(u8);

/// An [`u8`] that can be decremented. If it is decremented while it's at 0 the
/// value stays 0.
/// This represents a sound/delay timer in the context of CHIP-8.
impl Timer {
    /// Gets the current value.
    pub fn get(&self) -> u8 {
        self.0
    }

    /// Sets the current value
    pub fn set(&mut self, value: u8) {
        self.0 = value;
    }

    /// Decrements by one. If the value was 0 before decrementing the value
    /// stays 0.
    pub fn decrement(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decrements() {
        let mut timer = Timer::default();
        timer.set(10);
        for _ in 1..5 {
            timer.decrement();
        }
        assert_eq!(timer.get(), 6);
    }

    #[test]
    fn test_stays_zero() {
        let mut timer = Timer::default();
        timer.set(1);
        timer.decrement();
        timer.decrement();
        assert_eq!(timer.get(), 0);
    }
}
