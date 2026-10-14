use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback failover reservation must pass. Check returns its problems, or none.
pub trait FallbackFailoverReservationRule: std::fmt::Debug {
    fn check(&self, item: &FallbackFailoverReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservationRequiredCodeRule {
    pub message: String,
}

impl FallbackFailoverReservationRule for FallbackFailoverReservationRequiredCodeRule {
    fn check(&self, item: &FallbackFailoverReservation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservationNameLengthRule {
    pub max_length: i64,
}

impl FallbackFailoverReservationRule for FallbackFailoverReservationNameLengthRule {
    fn check(&self, item: &FallbackFailoverReservation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservationCodeLengthRule {
    pub max_length: i64,
}

impl FallbackFailoverReservationRule for FallbackFailoverReservationCodeLengthRule {
    fn check(&self, item: &FallbackFailoverReservation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservationUnitLengthRule {
    pub max_length: i64,
}

impl FallbackFailoverReservationRule for FallbackFailoverReservationUnitLengthRule {
    fn check(&self, item: &FallbackFailoverReservation) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackFailoverReservationRule for FallbackFailoverReservationMinimumRangeRule {
    fn check(&self, item: &FallbackFailoverReservation) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a fallback failover reservation.
#[derive(Debug, Clone)]
pub struct FallbackFailoverReservationValidator {
    pub rules: Vec<Rc<dyn FallbackFailoverReservationRule>>,
}

impl FallbackFailoverReservationValidator {
    pub fn validate(&self, item: &FallbackFailoverReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackFailoverReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackFailoverReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_failover_reservation_validator() -> FallbackFailoverReservationValidator {
    FallbackFailoverReservationValidator {
        rules: vec![
            Rc::new(FallbackFailoverReservationRequiredCodeRule {
                message: "fallback failover reservation needs an entry".to_string(),
            }) as Rc<dyn FallbackFailoverReservationRule>,
            Rc::new(FallbackFailoverReservationNameLengthRule { max_length: 120 }),
            Rc::new(FallbackFailoverReservationCodeLengthRule { max_length: 32 }),
            Rc::new(FallbackFailoverReservationUnitLengthRule { max_length: 120 }),
            Rc::new(FallbackFailoverReservationMinimumRangeRule { min: 4.0, max: 2533.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_failover_reservation_validator();
        let item = new_fallback_failover_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackFailoverReservationRequiredCodeRule {
            message: "fallback failover reservation needs an entry".to_string(),
        };
        let mut item = new_fallback_failover_reservation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackFailoverReservationNameLengthRule { max_length: 32 };
        let mut item = new_fallback_failover_reservation("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackFailoverReservationCodeLengthRule { max_length: 3600 };
        let mut item = new_fallback_failover_reservation("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = FallbackFailoverReservationUnitLengthRule { max_length: 255 };
        let mut item = new_fallback_failover_reservation("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FallbackFailoverReservationMinimumRangeRule { min: 5.2, max: 3124.3 };
        let mut item = new_fallback_failover_reservation("x", "y");
        item.minimum = 3125.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
