use concordium_std::{SchemaType, Serialize};

#[derive(Debug)]
pub enum ExchangeError {
    InvalidRate,
}

#[derive(Serialize, SchemaType, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rate {
    pub numerator:   u64,
    pub denominator: u64,
}

impl Default for Rate {
    fn default() -> Self {
        Self {
            numerator:   1,
            denominator: 1,
        }
    }
}

impl Rate {
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, ExchangeError> {
        if denominator == 0 {
            return Err(ExchangeError::InvalidRate);
        }

        Ok(Self {
            numerator,
            denominator,
        })
    }

    /// Is Less than `1`
    pub fn le_1(&self) -> bool { self.numerator <= self.denominator }

    /// is Valid
    pub fn is_valid(&self) -> bool { self.numerator != 0 && self.denominator != 0 }

    pub fn convert(&self, amount: &u64) -> Result<(u64, u64), ExchangeError> {
        let numerator = amount.checked_mul(self.numerator).ok_or(ExchangeError::InvalidRate)?;
        let converted_amount =
            numerator.checked_div(self.denominator).ok_or(ExchangeError::InvalidRate)?;
        let remainder =
            numerator.checked_rem(self.denominator).ok_or(ExchangeError::InvalidRate)?;
        Ok((converted_amount, remainder))
    }
}

#[cfg(test)]
mod tests {
    use super::Rate;

    #[test]
    fn calculate_amounts_rem() {
        let rate = Rate::new(1, 3).expect("valid rate");
        assert_eq!(rate.convert(&2).expect("valid amount"), (0, 2));
    }

    #[test]
    fn calculate_amounts() {
        let rate = Rate::new(1, 3).expect("valid rate");
        assert_eq!(rate.convert(&3).expect("valid amount"), (1, 0));
    }
}
