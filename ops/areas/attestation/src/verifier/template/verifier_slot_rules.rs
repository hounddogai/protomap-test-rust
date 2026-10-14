use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier slot must pass. Check returns its problems, or none.
pub trait VerifierSlotRule: std::fmt::Debug {
    fn check(&self, item: &VerifierSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlotCodePrefixRule {
    pub prefix: String,
}

impl VerifierSlotRule for VerifierSlotCodePrefixRule {
    fn check(&self, item: &VerifierSlot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlotRequiredTitleRule {
    pub message: String,
}

impl VerifierSlotRule for VerifierSlotRequiredTitleRule {
    fn check(&self, item: &VerifierSlot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlotScopeLengthRule {
    pub max_length: i64,
}

impl VerifierSlotRule for VerifierSlotScopeLengthRule {
    fn check(&self, item: &VerifierSlot) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier slot.
#[derive(Debug, Clone)]
pub struct VerifierSlotValidator {
    pub rules: Vec<Rc<dyn VerifierSlotRule>>,
}

impl VerifierSlotValidator {
    pub fn validate(&self, item: &VerifierSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_slot_validator() -> VerifierSlotValidator {
    VerifierSlotValidator {
        rules: vec![
            Rc::new(VerifierSlotCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn VerifierSlotRule>,
            Rc::new(VerifierSlotRequiredTitleRule {
                message: "verifier slot needs an entry".to_string(),
            }),
            Rc::new(VerifierSlotScopeLengthRule { max_length: 3600 }),
        ],
    }
}
