use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension adjustment must pass. Check returns its problems, or none.
pub trait SuspensionAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustmentTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl SuspensionAdjustmentRule for SuspensionAdjustmentTagsLimitRule {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl SuspensionAdjustmentRule for SuspensionAdjustmentCodePrefixRule {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustmentRequiredOwnerIdRule {
    pub message: String,
}

impl SuspensionAdjustmentRule for SuspensionAdjustmentRequiredOwnerIdRule {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustmentArchivedGuardRule {
    pub limit: i64,
}

impl SuspensionAdjustmentRule for SuspensionAdjustmentArchivedGuardRule {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String> {
        if item.archived && (item.window_days > self.limit) {
            return vec![format!(
                "window days {} exceeds {} while archived",
                item.window_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl SuspensionAdjustmentRule for SuspensionAdjustmentTitleLengthRule {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAdjustmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl SuspensionAdjustmentRule for SuspensionAdjustmentOwnerIdLengthRule {
    fn check(&self, item: &SuspensionAdjustment) -> Vec<String> {
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

/// Runs every rule of a suspension adjustment.
#[derive(Debug, Clone)]
pub struct SuspensionAdjustmentValidator {
    pub rules: Vec<Rc<dyn SuspensionAdjustmentRule>>,
}

impl SuspensionAdjustmentValidator {
    pub fn validate(&self, item: &SuspensionAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_adjustment_validator() -> SuspensionAdjustmentValidator {
    SuspensionAdjustmentValidator {
        rules: vec![
            Rc::new(SuspensionAdjustmentTagsLimitRule {
                max_count: 86400,
                banned: vec!["fallback".to_string()],
            }) as Rc<dyn SuspensionAdjustmentRule>,
            Rc::new(SuspensionAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SuspensionAdjustmentRequiredOwnerIdRule {
                message: "suspension adjustment needs this field".to_string(),
            }),
            Rc::new(SuspensionAdjustmentArchivedGuardRule { limit: 120 }),
            Rc::new(SuspensionAdjustmentTitleLengthRule { max_length: 120 }),
            Rc::new(SuspensionAdjustmentOwnerIdLengthRule { max_length: 64 }),
        ],
    }
}
