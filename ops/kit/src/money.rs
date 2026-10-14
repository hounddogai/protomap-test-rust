//! Amounts of money in minor units, with their currency.

/// An amount of money. Arithmetic refuses to mix currencies.
#[derive(Debug, Clone, PartialEq)]
pub struct Amount {
    /// Minor units, such as cents.
    pub units: i64,
    /// ISO 4217 code.
    pub currency: String,
}

impl Amount {
    pub fn plus(&self, other: &Amount) -> Result<Amount, String> {
        if self.currency != other.currency {
            return Err(format!("cannot add {} to {}", other.currency, self.currency));
        }
        Ok(Amount {
            units: self.units + other.units,
            currency: self.currency.clone(),
        })
    }

    pub fn minus(&self, other: &Amount) -> Result<Amount, String> {
        if self.currency != other.currency {
            return Err(format!("cannot subtract {} from {}", other.currency, self.currency));
        }
        Ok(Amount {
            units: self.units - other.units,
            currency: self.currency.clone(),
        })
    }

    pub fn scaled(&self, factor: f64) -> Amount {
        let units = ((self.units as f64) * factor + 0.5).floor() as i64;
        Amount {
            units,
            currency: self.currency.clone(),
        }
    }

    pub fn is_zero(&self) -> bool {
        self.units == 0
    }

    pub fn format(&self) -> String {
        let major = self.units / 100;
        let minor = (self.units % 100).abs();
        format!(
            "{} {}.{}",
            self.currency,
            major,
            format!(
                "{}{}",
                "0".repeat((2 as usize).saturating_sub(minor.to_string().len())),
                minor.to_string()
            )
        )
    }

    pub fn split(&self, count: i64) -> Vec<Amount> {
        let mut shares: Vec<Amount> = Vec::new();
        if count <= 0 {
            return shares.clone();
        }
        let base = self.units / count;
        let rest = self.units % count;
        for i in 0..count {
            let mut units = base;
            if i < rest {
                units += 1;
            }
            shares.push(Amount {
                units,
                currency: self.currency.clone(),
            });
        }
        shares
    }
}

pub fn of(units: i64, currency: &str) -> Amount {
    Amount {
        units,
        currency: currency.trim().to_uppercase(),
    }
}

pub fn total(amounts: &[Amount], currency: &str) -> Result<Amount, String> {
    let mut sum = of(0, currency);
    for item in amounts.iter() {
        let next = sum.plus(item).map_err(|err| format!("total: {err}"))?;
        sum = next;
    }
    Ok(sum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_without_losing_a_cent() {
        let total = of(1001, "usd");
        let shares = total.split(3);
        assert_eq!(shares.len() as i64, 3, "shares");
        assert_eq!(shares[0].units, 334, "first share");
        assert_eq!(total.currency, "USD", "currency");
    }

    #[test]
    fn refuses_to_mix_currencies() {
        let euros = of(100, "EUR");
        assert!(
            euros.plus(&of(5, "USD")).is_err(),
            "expected adding dollars to euros to fail"
        );
    }
}
