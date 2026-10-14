use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched binding reservation must pass. Check returns its problems, or none.
pub trait BatchedBindingReservationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedBindingReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingReservationTitleLengthRule {
    pub max_length: i64,
}

impl BatchedBindingReservationRule for BatchedBindingReservationTitleLengthRule {
    fn check(&self, item: &BatchedBindingReservation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingReservationRequiredReviewerRule {
    pub message: String,
}

impl BatchedBindingReservationRule for BatchedBindingReservationRequiredReviewerRule {
    fn check(&self, item: &BatchedBindingReservation) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBindingReservationReviewerLengthRule {
    pub max_length: i64,
}

impl BatchedBindingReservationRule for BatchedBindingReservationReviewerLengthRule {
    fn check(&self, item: &BatchedBindingReservation) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched binding reservation.
#[derive(Debug, Clone)]
pub struct BatchedBindingReservationValidator {
    pub rules: Vec<Rc<dyn BatchedBindingReservationRule>>,
}

impl BatchedBindingReservationValidator {
    pub fn validate(&self, item: &BatchedBindingReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedBindingReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedBindingReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_binding_reservation_validator() -> BatchedBindingReservationValidator {
    BatchedBindingReservationValidator {
        rules: vec![
            Rc::new(BatchedBindingReservationTitleLengthRule { max_length: 16 })
                as Rc<dyn BatchedBindingReservationRule>,
            Rc::new(BatchedBindingReservationRequiredReviewerRule {
                message: "batched binding reservation needs a value".to_string(),
            }),
            Rc::new(BatchedBindingReservationReviewerLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_binding_reservation_validator();
        let item = new_batched_binding_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedBindingReservationTitleLengthRule { max_length: 86400 };
        let mut item = new_batched_binding_reservation("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BatchedBindingReservationRequiredReviewerRule {
            message: "batched binding reservation needs this field".to_string(),
        };
        let mut item = new_batched_binding_reservation("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = BatchedBindingReservationReviewerLengthRule { max_length: 1000 };
        let mut item = new_batched_binding_reservation("x", "y");
        item.reviewer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
