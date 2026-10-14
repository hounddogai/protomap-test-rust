use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding threshold must pass. Check returns its problems, or none.
pub trait BindingThresholdRule: std::fmt::Debug {
    fn check(&self, item: &BindingThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingThresholdRequiredCodeRule {
    pub message: String,
}

impl BindingThresholdRule for BindingThresholdRequiredCodeRule {
    fn check(&self, item: &BindingThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingThresholdCodePrefixRule {
    pub prefix: String,
}

impl BindingThresholdRule for BindingThresholdCodePrefixRule {
    fn check(&self, item: &BindingThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingThresholdSourceLengthRule {
    pub max_length: i64,
}

impl BindingThresholdRule for BindingThresholdSourceLengthRule {
    fn check(&self, item: &BindingThreshold) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding threshold.
#[derive(Debug, Clone)]
pub struct BindingThresholdValidator {
    pub rules: Vec<Rc<dyn BindingThresholdRule>>,
}

impl BindingThresholdValidator {
    pub fn validate(&self, item: &BindingThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_threshold_validator() -> BindingThresholdValidator {
    BindingThresholdValidator {
        rules: vec![
            Rc::new(BindingThresholdRequiredCodeRule {
                message: "binding threshold needs an entry".to_string(),
            }) as Rc<dyn BindingThresholdRule>,
            Rc::new(BindingThresholdCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(BindingThresholdSourceLengthRule { max_length: 64 }),
        ],
    }
}
