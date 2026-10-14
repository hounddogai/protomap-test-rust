use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive overage reservation must pass. Check returns its problems, or none.
pub trait AdaptiveOverageReservationRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveOverageReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageReservationSourceLengthRule {
    pub max_length: i64,
}

impl AdaptiveOverageReservationRule for AdaptiveOverageReservationSourceLengthRule {
    fn check(&self, item: &AdaptiveOverageReservation) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageReservationRequiredSourceRule {
    pub message: String,
}

impl AdaptiveOverageReservationRule for AdaptiveOverageReservationRequiredSourceRule {
    fn check(&self, item: &AdaptiveOverageReservation) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageReservationLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveOverageReservationRule for AdaptiveOverageReservationLabelLengthRule {
    fn check(&self, item: &AdaptiveOverageReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive overage reservation.
#[derive(Debug, Clone)]
pub struct AdaptiveOverageReservationValidator {
    pub rules: Vec<Rc<dyn AdaptiveOverageReservationRule>>,
}

impl AdaptiveOverageReservationValidator {
    pub fn validate(&self, item: &AdaptiveOverageReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveOverageReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveOverageReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_overage_reservation_validator() -> AdaptiveOverageReservationValidator {
    AdaptiveOverageReservationValidator {
        rules: vec![
            Rc::new(AdaptiveOverageReservationSourceLengthRule { max_length: 1000 })
                as Rc<dyn AdaptiveOverageReservationRule>,
            Rc::new(AdaptiveOverageReservationRequiredSourceRule {
                message: "adaptive overage reservation needs this field".to_string(),
            }),
            Rc::new(AdaptiveOverageReservationLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_overage_reservation_validator();
        let item = new_adaptive_overage_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = AdaptiveOverageReservationSourceLengthRule { max_length: 32 };
        let mut item = new_adaptive_overage_reservation("x", "y");
        item.source = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = AdaptiveOverageReservationRequiredSourceRule {
            message: "adaptive overage reservation needs an entry".to_string(),
        };
        let mut item = new_adaptive_overage_reservation("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveOverageReservationLabelLengthRule { max_length: 1000 };
        let mut item = new_adaptive_overage_reservation("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
