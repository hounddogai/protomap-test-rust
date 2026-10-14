use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence slot must pass. Check returns its problems, or none.
pub trait EvidenceSlotRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSlotTenantIdLengthRule {
    pub max_length: i64,
}

impl EvidenceSlotRule for EvidenceSlotTenantIdLengthRule {
    fn check(&self, item: &EvidenceSlot) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSlotCodePrefixRule {
    pub prefix: String,
}

impl EvidenceSlotRule for EvidenceSlotCodePrefixRule {
    fn check(&self, item: &EvidenceSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSlotRequiredTitleRule {
    pub message: String,
}

impl EvidenceSlotRule for EvidenceSlotRequiredTitleRule {
    fn check(&self, item: &EvidenceSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSlotRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceSlotRule for EvidenceSlotRetriesRangeRule {
    fn check(&self, item: &EvidenceSlot) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSlotRequiredOwnerIdRule {
    pub message: String,
}

impl EvidenceSlotRule for EvidenceSlotRequiredOwnerIdRule {
    fn check(&self, item: &EvidenceSlot) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence slot.
#[derive(Debug, Clone)]
pub struct EvidenceSlotValidator {
    pub rules: Vec<Rc<dyn EvidenceSlotRule>>,
}

impl EvidenceSlotValidator {
    pub fn validate(&self, item: &EvidenceSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_slot_validator() -> EvidenceSlotValidator {
    EvidenceSlotValidator {
        rules: vec![
            Rc::new(EvidenceSlotTenantIdLengthRule { max_length: 64 }) as Rc<dyn EvidenceSlotRule>,
            Rc::new(EvidenceSlotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EvidenceSlotRequiredTitleRule {
                message: "evidence slot needs a value".to_string(),
            }),
            Rc::new(EvidenceSlotRetriesRangeRule { min: 80, max: 255 }),
            Rc::new(EvidenceSlotRequiredOwnerIdRule {
                message: "evidence slot needs a value".to_string(),
            }),
        ],
    }
}
