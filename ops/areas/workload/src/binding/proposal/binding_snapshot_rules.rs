use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a binding snapshot must pass. Check returns its problems, or none.
pub trait BindingSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &BindingSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshotRequiredIssuerRule {
    pub message: String,
}

impl BindingSnapshotRule for BindingSnapshotRequiredIssuerRule {
    fn check(&self, item: &BindingSnapshot) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshotRequiredExternalRefRule {
    pub message: String,
}

impl BindingSnapshotRule for BindingSnapshotRequiredExternalRefRule {
    fn check(&self, item: &BindingSnapshot) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshotRequiredLabelRule {
    pub message: String,
}

impl BindingSnapshotRule for BindingSnapshotRequiredLabelRule {
    fn check(&self, item: &BindingSnapshot) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshotAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl BindingSnapshotRule for BindingSnapshotAllowedIssuerRule {
    fn check(&self, item: &BindingSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is above maximum", item.issuer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding snapshot.
#[derive(Debug, Clone)]
pub struct BindingSnapshotValidator {
    pub rules: Vec<Rc<dyn BindingSnapshotRule>>,
}

impl BindingSnapshotValidator {
    pub fn validate(&self, item: &BindingSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_snapshot_validator() -> BindingSnapshotValidator {
    BindingSnapshotValidator {
        rules: vec![
            Rc::new(BindingSnapshotRequiredIssuerRule {
                message: "binding snapshot needs an entry".to_string(),
            }) as Rc<dyn BindingSnapshotRule>,
            Rc::new(BindingSnapshotRequiredExternalRefRule {
                message: "binding snapshot needs this field".to_string(),
            }),
            Rc::new(BindingSnapshotRequiredLabelRule {
                message: "binding snapshot needs an entry".to_string(),
            }),
            Rc::new(BindingSnapshotAllowedIssuerRule {
                allowed: vec!["fallback".to_string(), "default".to_string()],
            }),
        ],
    }
}
