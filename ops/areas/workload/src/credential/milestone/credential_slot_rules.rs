use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential slot must pass. Check returns its problems, or none.
pub trait CredentialSlotRule: std::fmt::Debug {
    fn check(&self, item: &CredentialSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSlotStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CredentialSlotRule for CredentialSlotStartsAtRangeRule {
    fn check(&self, item: &CredentialSlot) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSlotNameLengthRule {
    pub max_length: i64,
}

impl CredentialSlotRule for CredentialSlotNameLengthRule {
    fn check(&self, item: &CredentialSlot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialSlotRequiredNameRule {
    pub message: String,
}

impl CredentialSlotRule for CredentialSlotRequiredNameRule {
    fn check(&self, item: &CredentialSlot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential slot.
#[derive(Debug, Clone)]
pub struct CredentialSlotValidator {
    pub rules: Vec<Rc<dyn CredentialSlotRule>>,
}

impl CredentialSlotValidator {
    pub fn validate(&self, item: &CredentialSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CredentialSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_slot_validator() -> CredentialSlotValidator {
    CredentialSlotValidator {
        rules: vec![
            Rc::new(CredentialSlotStartsAtRangeRule { min: 1000, max: 1000 }) as Rc<dyn CredentialSlotRule>,
            Rc::new(CredentialSlotNameLengthRule { max_length: 1000 }),
            Rc::new(CredentialSlotRequiredNameRule {
                message: "credential slot needs an entry".to_string(),
            }),
        ],
    }
}
