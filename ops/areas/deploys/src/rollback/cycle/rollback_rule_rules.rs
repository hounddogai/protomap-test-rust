use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback rule must pass. Check returns its problems, or none.
pub trait RollbackRuleRule: std::fmt::Debug {
    fn check(&self, item: &RollbackRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleRequiredPhoneRule {
    pub message: String,
}

impl RollbackRuleRule for RollbackRuleRequiredPhoneRule {
    fn check(&self, item: &RollbackRule) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollbackRuleRule for RollbackRuleAllowedCodeRule {
    fn check(&self, item: &RollbackRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleCodePrefixRule {
    pub prefix: String,
}

impl RollbackRuleRule for RollbackRuleCodePrefixRule {
    fn check(&self, item: &RollbackRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleRequiredLabelRule {
    pub message: String,
}

impl RollbackRuleRule for RollbackRuleRequiredLabelRule {
    fn check(&self, item: &RollbackRule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRuleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackRuleRule for RollbackRuleSequenceRangeRule {
    fn check(&self, item: &RollbackRule) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollback rule.
#[derive(Debug, Clone)]
pub struct RollbackRuleValidator {
    pub rules: Vec<Rc<dyn RollbackRuleRule>>,
}

impl RollbackRuleValidator {
    pub fn validate(&self, item: &RollbackRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_rule_validator() -> RollbackRuleValidator {
    RollbackRuleValidator {
        rules: vec![
            Rc::new(RollbackRuleRequiredPhoneRule {
                message: "rollback rule needs an entry".to_string(),
            }) as Rc<dyn RollbackRuleRule>,
            Rc::new(RollbackRuleAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "lenient".to_string(), "legacy".to_string()],
            }),
            Rc::new(RollbackRuleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RollbackRuleRequiredLabelRule {
                message: "rollback rule needs an entry".to_string(),
            }),
            Rc::new(RollbackRuleSequenceRangeRule { min: 16, max: 16 }),
        ],
    }
}
