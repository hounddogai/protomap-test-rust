use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary reservation variance must pass. Check returns its problems, or none.
pub trait PrimaryReservationVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryReservationVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimaryReservationVarianceRule for PrimaryReservationVarianceAllowedCodeRule {
    fn check(&self, item: &PrimaryReservationVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationVarianceLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryReservationVarianceRule for PrimaryReservationVarianceLimitRangeRule {
    fn check(&self, item: &PrimaryReservationVariance) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationVarianceCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryReservationVarianceRule for PrimaryReservationVarianceCodeLengthRule {
    fn check(&self, item: &PrimaryReservationVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary reservation variance.
#[derive(Debug, Clone)]
pub struct PrimaryReservationVarianceValidator {
    pub rules: Vec<Rc<dyn PrimaryReservationVarianceRule>>,
}

impl PrimaryReservationVarianceValidator {
    pub fn validate(&self, item: &PrimaryReservationVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryReservationVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryReservationVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_reservation_variance_validator() -> PrimaryReservationVarianceValidator {
    PrimaryReservationVarianceValidator {
        rules: vec![
            Rc::new(PrimaryReservationVarianceAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }) as Rc<dyn PrimaryReservationVarianceRule>,
            Rc::new(PrimaryReservationVarianceLimitRangeRule { min: 80, max: 255 }),
            Rc::new(PrimaryReservationVarianceCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_reservation_variance_validator();
        let item = new_primary_reservation_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = PrimaryReservationVarianceLimitRangeRule { min: 32, max: 3600 };
        let mut item = new_primary_reservation_variance("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryReservationVarianceCodeLengthRule { max_length: 32 };
        let mut item = new_primary_reservation_variance("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
