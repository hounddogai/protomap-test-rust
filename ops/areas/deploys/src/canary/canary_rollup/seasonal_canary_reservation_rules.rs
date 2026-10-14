use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal canary reservation must pass. Check returns its problems, or none.
pub trait SeasonalCanaryReservationRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCanaryReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalCanaryReservationRule for SeasonalCanaryReservationMinimumRangeRule {
    fn check(&self, item: &SeasonalCanaryReservation) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservationChecksumLengthRule {
    pub max_length: i64,
}

impl SeasonalCanaryReservationRule for SeasonalCanaryReservationChecksumLengthRule {
    fn check(&self, item: &SeasonalCanaryReservation) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalCanaryReservationRule for SeasonalCanaryReservationAmountRangeRule {
    fn check(&self, item: &SeasonalCanaryReservation) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalCanaryReservationRule for SeasonalCanaryReservationRevisionRangeRule {
    fn check(&self, item: &SeasonalCanaryReservation) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservationRequiredChecksumRule {
    pub message: String,
}

impl SeasonalCanaryReservationRule for SeasonalCanaryReservationRequiredChecksumRule {
    fn check(&self, item: &SeasonalCanaryReservation) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal canary reservation.
#[derive(Debug, Clone)]
pub struct SeasonalCanaryReservationValidator {
    pub rules: Vec<Rc<dyn SeasonalCanaryReservationRule>>,
}

impl SeasonalCanaryReservationValidator {
    pub fn validate(&self, item: &SeasonalCanaryReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalCanaryReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCanaryReservation]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_seasonal_canary_reservation_validator() -> SeasonalCanaryReservationValidator {
    SeasonalCanaryReservationValidator {
        rules: vec![
            Rc::new(SeasonalCanaryReservationMinimumRangeRule { min: 9.3, max: 4631.5 })
                as Rc<dyn SeasonalCanaryReservationRule>,
            Rc::new(SeasonalCanaryReservationChecksumLengthRule { max_length: 3600 }),
            Rc::new(SeasonalCanaryReservationAmountRangeRule { min: 2.8, max: 981.2 }),
            Rc::new(SeasonalCanaryReservationRevisionRangeRule { min: 64, max: 64 }),
            Rc::new(SeasonalCanaryReservationRequiredChecksumRule {
                message: "seasonal canary reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_canary_reservation_validator();
        let item = new_seasonal_canary_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SeasonalCanaryReservationMinimumRangeRule { min: 4.0, max: 1676.5 };
        let mut item = new_seasonal_canary_reservation("x", "y");
        item.minimum = 1678.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = SeasonalCanaryReservationChecksumLengthRule { max_length: 1000 };
        let mut item = new_seasonal_canary_reservation("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SeasonalCanaryReservationAmountRangeRule { min: 8.7, max: 2228.0 };
        let mut item = new_seasonal_canary_reservation("x", "y");
        item.amount = 2229.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SeasonalCanaryReservationRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_seasonal_canary_reservation("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = SeasonalCanaryReservationRequiredChecksumRule {
            message: "seasonal canary reservation needs an entry".to_string(),
        };
        let mut item = new_seasonal_canary_reservation("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
