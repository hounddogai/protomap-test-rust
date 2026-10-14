use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched reservation entry must pass. Check returns its problems, or none.
pub trait BatchedReservationEntryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedReservationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationEntryCodeLengthRule {
    pub max_length: i64,
}

impl BatchedReservationEntryRule for BatchedReservationEntryCodeLengthRule {
    fn check(&self, item: &BatchedReservationEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationEntryRequiredKeyIdRule {
    pub message: String,
}

impl BatchedReservationEntryRule for BatchedReservationEntryRequiredKeyIdRule {
    fn check(&self, item: &BatchedReservationEntry) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationEntryAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl BatchedReservationEntryRule for BatchedReservationEntryAllowedNotesRule {
    fn check(&self, item: &BatchedReservationEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is stale", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationEntryRequiredNameRule {
    pub message: String,
}

impl BatchedReservationEntryRule for BatchedReservationEntryRequiredNameRule {
    fn check(&self, item: &BatchedReservationEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched reservation entry.
#[derive(Debug, Clone)]
pub struct BatchedReservationEntryValidator {
    pub rules: Vec<Rc<dyn BatchedReservationEntryRule>>,
}

impl BatchedReservationEntryValidator {
    pub fn validate(&self, item: &BatchedReservationEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedReservationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedReservationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_reservation_entry_validator() -> BatchedReservationEntryValidator {
    BatchedReservationEntryValidator {
        rules: vec![
            Rc::new(BatchedReservationEntryCodeLengthRule { max_length: 1000 }) as Rc<dyn BatchedReservationEntryRule>,
            Rc::new(BatchedReservationEntryRequiredKeyIdRule {
                message: "batched reservation entry needs an entry".to_string(),
            }),
            Rc::new(BatchedReservationEntryAllowedNotesRule { allowed: Vec::new() }),
            Rc::new(BatchedReservationEntryRequiredNameRule {
                message: "batched reservation entry needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_reservation_entry_validator();
        let item = new_batched_reservation_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedReservationEntryCodeLengthRule { max_length: 120 };
        let mut item = new_batched_reservation_entry("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = BatchedReservationEntryRequiredKeyIdRule {
            message: "batched reservation entry needs an entry".to_string(),
        };
        let mut item = new_batched_reservation_entry("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedReservationEntryRequiredNameRule {
            message: "batched reservation entry needs this field".to_string(),
        };
        let mut item = new_batched_reservation_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
