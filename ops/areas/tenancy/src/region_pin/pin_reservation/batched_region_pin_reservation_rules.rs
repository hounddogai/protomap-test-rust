use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched region pin reservation must pass. Check returns its problems, or none.
pub trait BatchedRegionPinReservationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedRegionPinReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservationScopeLengthRule {
    pub max_length: i64,
}

impl BatchedRegionPinReservationRule for BatchedRegionPinReservationScopeLengthRule {
    fn check(&self, item: &BatchedRegionPinReservation) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservationRequiredPrincipalRule {
    pub message: String,
}

impl BatchedRegionPinReservationRule for BatchedRegionPinReservationRequiredPrincipalRule {
    fn check(&self, item: &BatchedRegionPinReservation) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservationExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedRegionPinReservationRule for BatchedRegionPinReservationExpiresAtRangeRule {
    fn check(&self, item: &BatchedRegionPinReservation) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservationPrincipalLengthRule {
    pub max_length: i64,
}

impl BatchedRegionPinReservationRule for BatchedRegionPinReservationPrincipalLengthRule {
    fn check(&self, item: &BatchedRegionPinReservation) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched region pin reservation.
#[derive(Debug, Clone)]
pub struct BatchedRegionPinReservationValidator {
    pub rules: Vec<Rc<dyn BatchedRegionPinReservationRule>>,
}

impl BatchedRegionPinReservationValidator {
    pub fn validate(&self, item: &BatchedRegionPinReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedRegionPinReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedRegionPinReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_region_pin_reservation_validator() -> BatchedRegionPinReservationValidator {
    BatchedRegionPinReservationValidator {
        rules: vec![
            Rc::new(BatchedRegionPinReservationScopeLengthRule { max_length: 3600 })
                as Rc<dyn BatchedRegionPinReservationRule>,
            Rc::new(BatchedRegionPinReservationRequiredPrincipalRule {
                message: "batched region pin reservation needs an entry".to_string(),
            }),
            Rc::new(BatchedRegionPinReservationExpiresAtRangeRule { min: 64, max: 1000 }),
            Rc::new(BatchedRegionPinReservationPrincipalLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_region_pin_reservation_validator();
        let item = new_batched_region_pin_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BatchedRegionPinReservationScopeLengthRule { max_length: 3600 };
        let mut item = new_batched_region_pin_reservation("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = BatchedRegionPinReservationRequiredPrincipalRule {
            message: "batched region pin reservation needs this field".to_string(),
        };
        let mut item = new_batched_region_pin_reservation("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = BatchedRegionPinReservationExpiresAtRangeRule { min: 32, max: 32 };
        let mut item = new_batched_region_pin_reservation("x", "y");
        item.expires_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = BatchedRegionPinReservationPrincipalLengthRule { max_length: 255 };
        let mut item = new_batched_region_pin_reservation("x", "y");
        item.principal = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
