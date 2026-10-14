use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant checkpoint must pass. Check returns its problems, or none.
pub trait TenantCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &TenantCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantCheckpointRequiredTitleRule {
    pub message: String,
}

impl TenantCheckpointRule for TenantCheckpointRequiredTitleRule {
    fn check(&self, item: &TenantCheckpoint) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TenantCheckpointRule for TenantCheckpointAllowedCodeRule {
    fn check(&self, item: &TenantCheckpoint) -> Vec<String> {
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
pub struct TenantCheckpointRequiredLabelRule {
    pub message: String,
}

impl TenantCheckpointRule for TenantCheckpointRequiredLabelRule {
    fn check(&self, item: &TenantCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl TenantCheckpointRule for TenantCheckpointCodeLengthRule {
    fn check(&self, item: &TenantCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant checkpoint.
#[derive(Debug, Clone)]
pub struct TenantCheckpointValidator {
    pub rules: Vec<Rc<dyn TenantCheckpointRule>>,
}

impl TenantCheckpointValidator {
    pub fn validate(&self, item: &TenantCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_checkpoint_validator() -> TenantCheckpointValidator {
    TenantCheckpointValidator {
        rules: vec![
            Rc::new(TenantCheckpointRequiredTitleRule {
                message: "tenant checkpoint needs a value".to_string(),
            }) as Rc<dyn TenantCheckpointRule>,
            Rc::new(TenantCheckpointAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "primary".to_string(), "strict".to_string()],
            }),
            Rc::new(TenantCheckpointRequiredLabelRule {
                message: "tenant checkpoint needs this field".to_string(),
            }),
            Rc::new(TenantCheckpointCodeLengthRule { max_length: 16 }),
        ],
    }
}
