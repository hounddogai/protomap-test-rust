use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist milestone must pass. Check returns its problems, or none.
pub trait AllowlistMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMilestoneRequiredCodeRule {
    pub message: String,
}

impl AllowlistMilestoneRule for AllowlistMilestoneRequiredCodeRule {
    fn check(&self, item: &AllowlistMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMilestoneCodePrefixRule {
    pub prefix: String,
}

impl AllowlistMilestoneRule for AllowlistMilestoneCodePrefixRule {
    fn check(&self, item: &AllowlistMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistMilestoneRule for AllowlistMilestoneCodeLengthRule {
    fn check(&self, item: &AllowlistMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMilestoneRequiredTimezoneRule {
    pub message: String,
}

impl AllowlistMilestoneRule for AllowlistMilestoneRequiredTimezoneRule {
    fn check(&self, item: &AllowlistMilestone) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMilestoneRequiredNameRule {
    pub message: String,
}

impl AllowlistMilestoneRule for AllowlistMilestoneRequiredNameRule {
    fn check(&self, item: &AllowlistMilestone) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist milestone.
#[derive(Debug, Clone)]
pub struct AllowlistMilestoneValidator {
    pub rules: Vec<Rc<dyn AllowlistMilestoneRule>>,
}

impl AllowlistMilestoneValidator {
    pub fn validate(&self, item: &AllowlistMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_milestone_validator() -> AllowlistMilestoneValidator {
    AllowlistMilestoneValidator {
        rules: vec![
            Rc::new(AllowlistMilestoneRequiredCodeRule {
                message: "allowlist milestone needs an entry".to_string(),
            }) as Rc<dyn AllowlistMilestoneRule>,
            Rc::new(AllowlistMilestoneCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AllowlistMilestoneCodeLengthRule { max_length: 3600 }),
            Rc::new(AllowlistMilestoneRequiredTimezoneRule {
                message: "allowlist milestone needs a value".to_string(),
            }),
            Rc::new(AllowlistMilestoneRequiredNameRule {
                message: "allowlist milestone needs this field".to_string(),
            }),
        ],
    }
}
