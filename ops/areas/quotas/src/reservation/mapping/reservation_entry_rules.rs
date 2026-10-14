use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation entry must pass. Check returns its problems, or none.
pub trait ReservationEntryRule: std::fmt::Debug {
    fn check(&self, item: &ReservationEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationEntryLabelLengthRule {
    pub max_length: i64,
}

impl ReservationEntryRule for ReservationEntryLabelLengthRule {
    fn check(&self, item: &ReservationEntry) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationEntryCodePrefixRule {
    pub prefix: String,
}

impl ReservationEntryRule for ReservationEntryCodePrefixRule {
    fn check(&self, item: &ReservationEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationEntryPinnedGuardRule {
    pub limit: i64,
}

impl ReservationEntryRule for ReservationEntryPinnedGuardRule {
    fn check(&self, item: &ReservationEntry) -> Vec<String> {
        if item.pinned && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while pinned",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationEntryEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationEntryRule for ReservationEntryEndsAtRangeRule {
    fn check(&self, item: &ReservationEntry) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a reservation entry.
#[derive(Debug, Clone)]
pub struct ReservationEntryValidator {
    pub rules: Vec<Rc<dyn ReservationEntryRule>>,
}

impl ReservationEntryValidator {
    pub fn validate(&self, item: &ReservationEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_entry_validator() -> ReservationEntryValidator {
    ReservationEntryValidator {
        rules: vec![
            Rc::new(ReservationEntryLabelLengthRule { max_length: 120 }) as Rc<dyn ReservationEntryRule>,
            Rc::new(ReservationEntryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ReservationEntryPinnedGuardRule { limit: 86400 }),
            Rc::new(ReservationEntryEndsAtRangeRule { min: 80, max: 255 }),
        ],
    }
}
