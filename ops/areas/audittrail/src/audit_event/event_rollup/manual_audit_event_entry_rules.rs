use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual audit event entry must pass. Check returns its problems, or none.
pub trait ManualAuditEventEntryRule: std::fmt::Debug {
    fn check(&self, item: &ManualAuditEventEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntryRequiredUnitRule {
    pub message: String,
}

impl ManualAuditEventEntryRule for ManualAuditEventEntryRequiredUnitRule {
    fn check(&self, item: &ManualAuditEventEntry) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntryRequiredTitleRule {
    pub message: String,
}

impl ManualAuditEventEntryRule for ManualAuditEventEntryRequiredTitleRule {
    fn check(&self, item: &ManualAuditEventEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntryCodePrefixRule {
    pub prefix: String,
}

impl ManualAuditEventEntryRule for ManualAuditEventEntryCodePrefixRule {
    fn check(&self, item: &ManualAuditEventEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntryTitleLengthRule {
    pub max_length: i64,
}

impl ManualAuditEventEntryRule for ManualAuditEventEntryTitleLengthRule {
    fn check(&self, item: &ManualAuditEventEntry) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAuditEventEntryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualAuditEventEntryRule for ManualAuditEventEntryAmountRangeRule {
    fn check(&self, item: &ManualAuditEventEntry) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual audit event entry.
#[derive(Debug, Clone)]
pub struct ManualAuditEventEntryValidator {
    pub rules: Vec<Rc<dyn ManualAuditEventEntryRule>>,
}

impl ManualAuditEventEntryValidator {
    pub fn validate(&self, item: &ManualAuditEventEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAuditEventEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAuditEventEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_audit_event_entry_validator() -> ManualAuditEventEntryValidator {
    ManualAuditEventEntryValidator {
        rules: vec![
            Rc::new(ManualAuditEventEntryRequiredUnitRule {
                message: "manual audit event entry needs this field".to_string(),
            }) as Rc<dyn ManualAuditEventEntryRule>,
            Rc::new(ManualAuditEventEntryRequiredTitleRule {
                message: "manual audit event entry needs this field".to_string(),
            }),
            Rc::new(ManualAuditEventEntryCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ManualAuditEventEntryTitleLengthRule { max_length: 255 }),
            Rc::new(ManualAuditEventEntryAmountRangeRule { min: 4.6, max: 784.0 }),
        ],
    }
}
