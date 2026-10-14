use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy allowlist checkpoint must pass. Check returns its problems, or none.
pub trait LegacyAllowlistCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistCheckpointRule for LegacyAllowlistCheckpointCodeLengthRule {
    fn check(&self, item: &LegacyAllowlistCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistCheckpointActiveGuardRule {
    pub limit: i64,
}

impl LegacyAllowlistCheckpointRule for LegacyAllowlistCheckpointActiveGuardRule {
    fn check(&self, item: &LegacyAllowlistCheckpoint) -> Vec<String> {
        if item.active && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while active",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistCheckpointAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacyAllowlistCheckpointRule for LegacyAllowlistCheckpointAllowedTitleRule {
    fn check(&self, item: &LegacyAllowlistCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is unknown", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistCheckpointRequiredTitleRule {
    pub message: String,
}

impl LegacyAllowlistCheckpointRule for LegacyAllowlistCheckpointRequiredTitleRule {
    fn check(&self, item: &LegacyAllowlistCheckpoint) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist checkpoint.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistCheckpointValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistCheckpointRule>>,
}

impl LegacyAllowlistCheckpointValidator {
    pub fn validate(&self, item: &LegacyAllowlistCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAllowlistCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_checkpoint_validator() -> LegacyAllowlistCheckpointValidator {
    LegacyAllowlistCheckpointValidator {
        rules: vec![
            Rc::new(LegacyAllowlistCheckpointCodeLengthRule { max_length: 16 })
                as Rc<dyn LegacyAllowlistCheckpointRule>,
            Rc::new(LegacyAllowlistCheckpointActiveGuardRule { limit: 64 }),
            Rc::new(LegacyAllowlistCheckpointAllowedTitleRule {
                allowed: vec!["manual".to_string(), "batched".to_string(), "weighted".to_string()],
            }),
            Rc::new(LegacyAllowlistCheckpointRequiredTitleRule {
                message: "legacy allowlist checkpoint needs an entry".to_string(),
            }),
        ],
    }
}
