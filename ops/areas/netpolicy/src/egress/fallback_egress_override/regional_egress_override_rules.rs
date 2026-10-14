use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional egress override must pass. Check returns its problems, or none.
pub trait RegionalEgressOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RegionalEgressOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressOverrideRequiredHostRule {
    pub message: String,
}

impl RegionalEgressOverrideRule for RegionalEgressOverrideRequiredHostRule {
    fn check(&self, item: &RegionalEgressOverride) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressOverrideTitleLengthRule {
    pub max_length: i64,
}

impl RegionalEgressOverrideRule for RegionalEgressOverrideTitleLengthRule {
    fn check(&self, item: &RegionalEgressOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressOverrideRequiredTitleRule {
    pub message: String,
}

impl RegionalEgressOverrideRule for RegionalEgressOverrideRequiredTitleRule {
    fn check(&self, item: &RegionalEgressOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RegionalEgressOverrideRule for RegionalEgressOverrideCodeLengthRule {
    fn check(&self, item: &RegionalEgressOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional egress override.
#[derive(Debug, Clone)]
pub struct RegionalEgressOverrideValidator {
    pub rules: Vec<Rc<dyn RegionalEgressOverrideRule>>,
}

impl RegionalEgressOverrideValidator {
    pub fn validate(&self, item: &RegionalEgressOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalEgressOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalEgressOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_egress_override_validator() -> RegionalEgressOverrideValidator {
    RegionalEgressOverrideValidator {
        rules: vec![
            Rc::new(RegionalEgressOverrideRequiredHostRule {
                message: "regional egress override needs this field".to_string(),
            }) as Rc<dyn RegionalEgressOverrideRule>,
            Rc::new(RegionalEgressOverrideTitleLengthRule { max_length: 120 }),
            Rc::new(RegionalEgressOverrideRequiredTitleRule {
                message: "regional egress override needs this field".to_string(),
            }),
            Rc::new(RegionalEgressOverrideCodeLengthRule { max_length: 32 }),
        ],
    }
}
