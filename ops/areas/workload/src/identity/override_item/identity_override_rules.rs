use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity override must pass. Check returns its problems, or none.
pub trait IdentityOverrideRule: std::fmt::Debug {
    fn check(&self, item: &IdentityOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverrideRequiredNameRule {
    pub message: String,
}

impl IdentityOverrideRule for IdentityOverrideRequiredNameRule {
    fn check(&self, item: &IdentityOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverrideNameLengthRule {
    pub max_length: i64,
}

impl IdentityOverrideRule for IdentityOverrideNameLengthRule {
    fn check(&self, item: &IdentityOverride) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverrideRequiredLabelRule {
    pub message: String,
}

impl IdentityOverrideRule for IdentityOverrideRequiredLabelRule {
    fn check(&self, item: &IdentityOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity override.
#[derive(Debug, Clone)]
pub struct IdentityOverrideValidator {
    pub rules: Vec<Rc<dyn IdentityOverrideRule>>,
}

impl IdentityOverrideValidator {
    pub fn validate(&self, item: &IdentityOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IdentityOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_override_validator() -> IdentityOverrideValidator {
    IdentityOverrideValidator {
        rules: vec![
            Rc::new(IdentityOverrideRequiredNameRule {
                message: "identity override needs a value".to_string(),
            }) as Rc<dyn IdentityOverrideRule>,
            Rc::new(IdentityOverrideNameLengthRule { max_length: 86400 }),
            Rc::new(IdentityOverrideRequiredLabelRule {
                message: "identity override needs an entry".to_string(),
            }),
        ],
    }
}
