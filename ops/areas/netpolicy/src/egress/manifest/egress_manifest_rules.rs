use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress manifest must pass. Check returns its problems, or none.
pub trait EgressManifestRule: std::fmt::Debug {
    fn check(&self, item: &EgressManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifestKeyIdLengthRule {
    pub max_length: i64,
}

impl EgressManifestRule for EgressManifestKeyIdLengthRule {
    fn check(&self, item: &EgressManifest) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifestRequiredCodeRule {
    pub message: String,
}

impl EgressManifestRule for EgressManifestRequiredCodeRule {
    fn check(&self, item: &EgressManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifestRequiredLabelRule {
    pub message: String,
}

impl EgressManifestRule for EgressManifestRequiredLabelRule {
    fn check(&self, item: &EgressManifest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifestCodePrefixRule {
    pub prefix: String,
}

impl EgressManifestRule for EgressManifestCodePrefixRule {
    fn check(&self, item: &EgressManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressManifestBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressManifestRule for EgressManifestBudgetRangeRule {
    fn check(&self, item: &EgressManifest) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a egress manifest.
#[derive(Debug, Clone)]
pub struct EgressManifestValidator {
    pub rules: Vec<Rc<dyn EgressManifestRule>>,
}

impl EgressManifestValidator {
    pub fn validate(&self, item: &EgressManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_manifest_validator() -> EgressManifestValidator {
    EgressManifestValidator {
        rules: vec![
            Rc::new(EgressManifestKeyIdLengthRule { max_length: 64 }) as Rc<dyn EgressManifestRule>,
            Rc::new(EgressManifestRequiredCodeRule {
                message: "egress manifest needs this field".to_string(),
            }),
            Rc::new(EgressManifestRequiredLabelRule {
                message: "egress manifest needs a value".to_string(),
            }),
            Rc::new(EgressManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(EgressManifestBudgetRangeRule { min: 1.6, max: 3383.4 }),
        ],
    }
}
