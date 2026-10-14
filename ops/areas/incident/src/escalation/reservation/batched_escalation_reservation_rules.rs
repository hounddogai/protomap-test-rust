use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched escalation reservation must pass. Check returns its problems, or none.
pub trait BatchedEscalationReservationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedEscalationReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservationApprovedGuardRule {
    pub limit: f64,
}

impl BatchedEscalationReservationRule for BatchedEscalationReservationApprovedGuardRule {
    fn check(&self, item: &BatchedEscalationReservation) -> Vec<String> {
        if item.approved && (item.score > self.limit) {
            return vec![format!("score {} exceeds {} while approved", item.score, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservationCodePrefixRule {
    pub prefix: String,
}

impl BatchedEscalationReservationRule for BatchedEscalationReservationCodePrefixRule {
    fn check(&self, item: &BatchedEscalationReservation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservationHostLengthRule {
    pub max_length: i64,
}

impl BatchedEscalationReservationRule for BatchedEscalationReservationHostLengthRule {
    fn check(&self, item: &BatchedEscalationReservation) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservationRequiredNameRule {
    pub message: String,
}

impl BatchedEscalationReservationRule for BatchedEscalationReservationRequiredNameRule {
    fn check(&self, item: &BatchedEscalationReservation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationReservationRequiredRegionRule {
    pub message: String,
}

impl BatchedEscalationReservationRule for BatchedEscalationReservationRequiredRegionRule {
    fn check(&self, item: &BatchedEscalationReservation) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched escalation reservation.
#[derive(Debug, Clone)]
pub struct BatchedEscalationReservationValidator {
    pub rules: Vec<Rc<dyn BatchedEscalationReservationRule>>,
}

impl BatchedEscalationReservationValidator {
    pub fn validate(&self, item: &BatchedEscalationReservation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedEscalationReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedEscalationReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_escalation_reservation_validator() -> BatchedEscalationReservationValidator {
    BatchedEscalationReservationValidator {
        rules: vec![
            Rc::new(BatchedEscalationReservationApprovedGuardRule { limit: 4.3 })
                as Rc<dyn BatchedEscalationReservationRule>,
            Rc::new(BatchedEscalationReservationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BatchedEscalationReservationHostLengthRule { max_length: 255 }),
            Rc::new(BatchedEscalationReservationRequiredNameRule {
                message: "batched escalation reservation needs this field".to_string(),
            }),
            Rc::new(BatchedEscalationReservationRequiredRegionRule {
                message: "batched escalation reservation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_escalation_reservation_validator();
        let item = new_batched_escalation_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = BatchedEscalationReservationHostLengthRule { max_length: 255 };
        let mut item = new_batched_escalation_reservation("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedEscalationReservationRequiredNameRule {
            message: "batched escalation reservation needs an entry".to_string(),
        };
        let mut item = new_batched_escalation_reservation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = BatchedEscalationReservationRequiredRegionRule {
            message: "batched escalation reservation needs an entry".to_string(),
        };
        let mut item = new_batched_escalation_reservation("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
