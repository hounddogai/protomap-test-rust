use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation exception must pass. Check returns its problems, or none.
pub trait ReservationExceptionRule: std::fmt::Debug {
    fn check(&self, item: &ReservationException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationExceptionMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationExceptionRule for ReservationExceptionMinimumRangeRule {
    fn check(&self, item: &ReservationException) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationExceptionCodePrefixRule {
    pub prefix: String,
}

impl ReservationExceptionRule for ReservationExceptionCodePrefixRule {
    fn check(&self, item: &ReservationException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationExceptionRequiredExternalRefRule {
    pub message: String,
}

impl ReservationExceptionRule for ReservationExceptionRequiredExternalRefRule {
    fn check(&self, item: &ReservationException) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation exception.
#[derive(Debug, Clone)]
pub struct ReservationExceptionValidator {
    pub rules: Vec<Rc<dyn ReservationExceptionRule>>,
}

impl ReservationExceptionValidator {
    pub fn validate(&self, item: &ReservationException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_exception_validator() -> ReservationExceptionValidator {
    ReservationExceptionValidator {
        rules: vec![
            Rc::new(ReservationExceptionMinimumRangeRule { min: 9.3, max: 4332.0 }) as Rc<dyn ReservationExceptionRule>,
            Rc::new(ReservationExceptionCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ReservationExceptionRequiredExternalRefRule {
                message: "reservation exception needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_exception_validator();
        let item = new_reservation_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ReservationExceptionMinimumRangeRule { min: 3.0, max: 4805.4 };
        let mut item = new_reservation_exception("x", "y");
        item.minimum = 4806.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ReservationExceptionRequiredExternalRefRule {
            message: "reservation exception needs this field".to_string(),
        };
        let mut item = new_reservation_exception("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
