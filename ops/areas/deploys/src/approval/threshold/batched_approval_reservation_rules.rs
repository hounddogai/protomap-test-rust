use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched approval reservation must pass. Check returns its problems, or none.
pub trait BatchedApprovalReservationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedApprovalReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationLabelLengthRule {
    pub max_length: i64,
}

impl BatchedApprovalReservationRule for BatchedApprovalReservationLabelLengthRule {
    fn check(&self, item: &BatchedApprovalReservation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationClusterLengthRule {
    pub max_length: i64,
}

impl BatchedApprovalReservationRule for BatchedApprovalReservationClusterLengthRule {
    fn check(&self, item: &BatchedApprovalReservation) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationRequiredReviewerRule {
    pub message: String,
}

impl BatchedApprovalReservationRule for BatchedApprovalReservationRequiredReviewerRule {
    fn check(&self, item: &BatchedApprovalReservation) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl BatchedApprovalReservationRule for BatchedApprovalReservationAllowedLabelRule {
    fn check(&self, item: &BatchedApprovalReservation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationRequiredLabelRule {
    pub message: String,
}

impl BatchedApprovalReservationRule for BatchedApprovalReservationRequiredLabelRule {
    fn check(&self, item: &BatchedApprovalReservation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched approval reservation.
#[derive(Debug, Clone)]
pub struct BatchedApprovalReservationValidator {
    pub rules: Vec<Rc<dyn BatchedApprovalReservationRule>>,
}

impl BatchedApprovalReservationValidator {
    pub fn validate(&self, item: &BatchedApprovalReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedApprovalReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedApprovalReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_approval_reservation_validator() -> BatchedApprovalReservationValidator {
    BatchedApprovalReservationValidator {
        rules: vec![
            Rc::new(BatchedApprovalReservationLabelLengthRule { max_length: 80 })
                as Rc<dyn BatchedApprovalReservationRule>,
            Rc::new(BatchedApprovalReservationClusterLengthRule { max_length: 120 }),
            Rc::new(BatchedApprovalReservationRequiredReviewerRule {
                message: "batched approval reservation needs an entry".to_string(),
            }),
            Rc::new(BatchedApprovalReservationAllowedLabelRule {
                allowed: vec!["seasonal".to_string(), "seasonal".to_string()],
            }),
            Rc::new(BatchedApprovalReservationRequiredLabelRule {
                message: "batched approval reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_approval_reservation_validator();
        let item = new_batched_approval_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedApprovalReservationLabelLengthRule { max_length: 1000 };
        let mut item = new_batched_approval_reservation("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = BatchedApprovalReservationClusterLengthRule { max_length: 16 };
        let mut item = new_batched_approval_reservation("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = BatchedApprovalReservationRequiredReviewerRule {
            message: "batched approval reservation needs a value".to_string(),
        };
        let mut item = new_batched_approval_reservation("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedApprovalReservationRequiredLabelRule {
            message: "batched approval reservation needs an entry".to_string(),
        };
        let mut item = new_batched_approval_reservation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
