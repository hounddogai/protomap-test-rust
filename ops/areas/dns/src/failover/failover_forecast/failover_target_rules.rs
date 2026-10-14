use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover target must pass. Check returns its problems, or none.
pub trait FailoverTargetRule: std::fmt::Debug {
    fn check(&self, item: &FailoverTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTargetRequiredCodeRule {
    pub message: String,
}

impl FailoverTargetRule for FailoverTargetRequiredCodeRule {
    fn check(&self, item: &FailoverTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTargetCodeLengthRule {
    pub max_length: i64,
}

impl FailoverTargetRule for FailoverTargetCodeLengthRule {
    fn check(&self, item: &FailoverTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTargetRequiredSourceRule {
    pub message: String,
}

impl FailoverTargetRule for FailoverTargetRequiredSourceRule {
    fn check(&self, item: &FailoverTarget) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTargetRequiredLabelRule {
    pub message: String,
}

impl FailoverTargetRule for FailoverTargetRequiredLabelRule {
    fn check(&self, item: &FailoverTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover target.
#[derive(Debug, Clone)]
pub struct FailoverTargetValidator {
    pub rules: Vec<Rc<dyn FailoverTargetRule>>,
}

impl FailoverTargetValidator {
    pub fn validate(&self, item: &FailoverTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_target_validator() -> FailoverTargetValidator {
    FailoverTargetValidator {
        rules: vec![
            Rc::new(FailoverTargetRequiredCodeRule {
                message: "failover target needs a value".to_string(),
            }) as Rc<dyn FailoverTargetRule>,
            Rc::new(FailoverTargetCodeLengthRule { max_length: 80 }),
            Rc::new(FailoverTargetRequiredSourceRule {
                message: "failover target needs a value".to_string(),
            }),
            Rc::new(FailoverTargetRequiredLabelRule {
                message: "failover target needs this field".to_string(),
            }),
        ],
    }
}
