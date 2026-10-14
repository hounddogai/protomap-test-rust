use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval rollup must pass. Check returns its problems, or none.
pub trait ApprovalRollupRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRollupCodePrefixRule {
    pub prefix: String,
}

impl ApprovalRollupRule for ApprovalRollupCodePrefixRule {
    fn check(&self, item: &ApprovalRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRollupSegmentLengthRule {
    pub max_length: i64,
}

impl ApprovalRollupRule for ApprovalRollupSegmentLengthRule {
    fn check(&self, item: &ApprovalRollup) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRollupRequiredSegmentRule {
    pub message: String,
}

impl ApprovalRollupRule for ApprovalRollupRequiredSegmentRule {
    fn check(&self, item: &ApprovalRollup) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRollupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalRollupRule for ApprovalRollupPortRangeRule {
    fn check(&self, item: &ApprovalRollup) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a approval rollup.
#[derive(Debug, Clone)]
pub struct ApprovalRollupValidator {
    pub rules: Vec<Rc<dyn ApprovalRollupRule>>,
}

impl ApprovalRollupValidator {
    pub fn validate(&self, item: &ApprovalRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_rollup_validator() -> ApprovalRollupValidator {
    ApprovalRollupValidator {
        rules: vec![
            Rc::new(ApprovalRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn ApprovalRollupRule>,
            Rc::new(ApprovalRollupSegmentLengthRule { max_length: 64 }),
            Rc::new(ApprovalRollupRequiredSegmentRule {
                message: "approval rollup needs an entry".to_string(),
            }),
            Rc::new(ApprovalRollupPortRangeRule { min: 32, max: 120 }),
        ],
    }
}
