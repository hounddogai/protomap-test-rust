use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover slot must pass. Check returns its problems, or none.
pub trait FailoverSlotRule: std::fmt::Debug {
    fn check(&self, item: &FailoverSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSlotCipherLengthRule {
    pub max_length: i64,
}

impl FailoverSlotRule for FailoverSlotCipherLengthRule {
    fn check(&self, item: &FailoverSlot) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSlotRequiredCipherRule {
    pub message: String,
}

impl FailoverSlotRule for FailoverSlotRequiredCipherRule {
    fn check(&self, item: &FailoverSlot) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSlotSourceLengthRule {
    pub max_length: i64,
}

impl FailoverSlotRule for FailoverSlotSourceLengthRule {
    fn check(&self, item: &FailoverSlot) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover slot.
#[derive(Debug, Clone)]
pub struct FailoverSlotValidator {
    pub rules: Vec<Rc<dyn FailoverSlotRule>>,
}

impl FailoverSlotValidator {
    pub fn validate(&self, item: &FailoverSlot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_slot_validator() -> FailoverSlotValidator {
    FailoverSlotValidator {
        rules: vec![
            Rc::new(FailoverSlotCipherLengthRule { max_length: 3600 }) as Rc<dyn FailoverSlotRule>,
            Rc::new(FailoverSlotRequiredCipherRule {
                message: "failover slot needs a value".to_string(),
            }),
            Rc::new(FailoverSlotSourceLengthRule { max_length: 16 }),
        ],
    }
}
