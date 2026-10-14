use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled suspension override must pass. Check returns its problems, or none.
pub trait ScheduledSuspensionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSuspensionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverrideRequiredPrincipalRule {
    pub message: String,
}

impl ScheduledSuspensionOverrideRule for ScheduledSuspensionOverrideRequiredPrincipalRule {
    fn check(&self, item: &ScheduledSuspensionOverride) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverrideRequiredOwnerIdRule {
    pub message: String,
}

impl ScheduledSuspensionOverrideRule for ScheduledSuspensionOverrideRequiredOwnerIdRule {
    fn check(&self, item: &ScheduledSuspensionOverride) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverrideOwnerIdLengthRule {
    pub max_length: i64,
}

impl ScheduledSuspensionOverrideRule for ScheduledSuspensionOverrideOwnerIdLengthRule {
    fn check(&self, item: &ScheduledSuspensionOverride) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverrideArchivedGuardRule {
    pub limit: i64,
}

impl ScheduledSuspensionOverrideRule for ScheduledSuspensionOverrideArchivedGuardRule {
    fn check(&self, item: &ScheduledSuspensionOverride) -> Vec<String> {
        if item.archived && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while archived",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSuspensionOverrideRequiredCodeRule {
    pub message: String,
}

impl ScheduledSuspensionOverrideRule for ScheduledSuspensionOverrideRequiredCodeRule {
    fn check(&self, item: &ScheduledSuspensionOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled suspension override.
#[derive(Debug, Clone)]
pub struct ScheduledSuspensionOverrideValidator {
    pub rules: Vec<Rc<dyn ScheduledSuspensionOverrideRule>>,
}

impl ScheduledSuspensionOverrideValidator {
    pub fn validate(&self, item: &ScheduledSuspensionOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSuspensionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSuspensionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_suspension_override_validator() -> ScheduledSuspensionOverrideValidator {
    ScheduledSuspensionOverrideValidator {
        rules: vec![
            Rc::new(ScheduledSuspensionOverrideRequiredPrincipalRule {
                message: "scheduled suspension override needs an entry".to_string(),
            }) as Rc<dyn ScheduledSuspensionOverrideRule>,
            Rc::new(ScheduledSuspensionOverrideRequiredOwnerIdRule {
                message: "scheduled suspension override needs a value".to_string(),
            }),
            Rc::new(ScheduledSuspensionOverrideOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(ScheduledSuspensionOverrideArchivedGuardRule { limit: 80 }),
            Rc::new(ScheduledSuspensionOverrideRequiredCodeRule {
                message: "scheduled suspension override needs this field".to_string(),
            }),
        ],
    }
}
