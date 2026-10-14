use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling evidence slot must pass. Check returns its problems, or none.
pub trait RollingEvidenceSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollingEvidenceSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceSlotCodePrefixRule {
    pub prefix: String,
}

impl RollingEvidenceSlotRule for RollingEvidenceSlotCodePrefixRule {
    fn check(&self, item: &RollingEvidenceSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceSlotSourceLengthRule {
    pub max_length: i64,
}

impl RollingEvidenceSlotRule for RollingEvidenceSlotSourceLengthRule {
    fn check(&self, item: &RollingEvidenceSlot) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceSlotRequiredContactEmailRule {
    pub message: String,
}

impl RollingEvidenceSlotRule for RollingEvidenceSlotRequiredContactEmailRule {
    fn check(&self, item: &RollingEvidenceSlot) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceSlotRequiredCodeRule {
    pub message: String,
}

impl RollingEvidenceSlotRule for RollingEvidenceSlotRequiredCodeRule {
    fn check(&self, item: &RollingEvidenceSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling evidence slot.
#[derive(Debug, Clone)]
pub struct RollingEvidenceSlotValidator {
    pub rules: Vec<Rc<dyn RollingEvidenceSlotRule>>,
}

impl RollingEvidenceSlotValidator {
    pub fn validate(&self, item: &RollingEvidenceSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingEvidenceSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEvidenceSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_evidence_slot_validator() -> RollingEvidenceSlotValidator {
    RollingEvidenceSlotValidator {
        rules: vec![
            Rc::new(RollingEvidenceSlotCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollingEvidenceSlotRule>,
            Rc::new(RollingEvidenceSlotSourceLengthRule { max_length: 3600 }),
            Rc::new(RollingEvidenceSlotRequiredContactEmailRule {
                message: "rolling evidence slot needs a value".to_string(),
            }),
            Rc::new(RollingEvidenceSlotRequiredCodeRule {
                message: "rolling evidence slot needs an entry".to_string(),
            }),
        ],
    }
}
