use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling isolation policy must pass. Check returns its problems, or none.
pub trait RollingIsolationPolicyRule: std::fmt::Debug {
    fn check(&self, item: &RollingIsolationPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicyTitleLengthRule {
    pub max_length: i64,
}

impl RollingIsolationPolicyRule for RollingIsolationPolicyTitleLengthRule {
    fn check(&self, item: &RollingIsolationPolicy) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicyNotesLengthRule {
    pub max_length: i64,
}

impl RollingIsolationPolicyRule for RollingIsolationPolicyNotesLengthRule {
    fn check(&self, item: &RollingIsolationPolicy) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicyRequiredCodeRule {
    pub message: String,
}

impl RollingIsolationPolicyRule for RollingIsolationPolicyRequiredCodeRule {
    fn check(&self, item: &RollingIsolationPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicyCodeLengthRule {
    pub max_length: i64,
}

impl RollingIsolationPolicyRule for RollingIsolationPolicyCodeLengthRule {
    fn check(&self, item: &RollingIsolationPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationPolicyRequiredTitleRule {
    pub message: String,
}

impl RollingIsolationPolicyRule for RollingIsolationPolicyRequiredTitleRule {
    fn check(&self, item: &RollingIsolationPolicy) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling isolation policy.
#[derive(Debug, Clone)]
pub struct RollingIsolationPolicyValidator {
    pub rules: Vec<Rc<dyn RollingIsolationPolicyRule>>,
}

impl RollingIsolationPolicyValidator {
    pub fn validate(&self, item: &RollingIsolationPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingIsolationPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingIsolationPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_isolation_policy_validator() -> RollingIsolationPolicyValidator {
    RollingIsolationPolicyValidator {
        rules: vec![
            Rc::new(RollingIsolationPolicyTitleLengthRule { max_length: 3600 }) as Rc<dyn RollingIsolationPolicyRule>,
            Rc::new(RollingIsolationPolicyNotesLengthRule { max_length: 120 }),
            Rc::new(RollingIsolationPolicyRequiredCodeRule {
                message: "rolling isolation policy needs a value".to_string(),
            }),
            Rc::new(RollingIsolationPolicyCodeLengthRule { max_length: 64 }),
            Rc::new(RollingIsolationPolicyRequiredTitleRule {
                message: "rolling isolation policy needs an entry".to_string(),
            }),
        ],
    }
}
