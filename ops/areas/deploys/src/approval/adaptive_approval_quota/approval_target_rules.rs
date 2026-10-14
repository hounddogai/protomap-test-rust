use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval target must pass. Check returns its problems, or none.
pub trait ApprovalTargetRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetLabelLengthRule {
    pub max_length: i64,
}

impl ApprovalTargetRule for ApprovalTargetLabelLengthRule {
    fn check(&self, item: &ApprovalTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetRequiredAudienceRule {
    pub message: String,
}

impl ApprovalTargetRule for ApprovalTargetRequiredAudienceRule {
    fn check(&self, item: &ApprovalTarget) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetRequiredLabelRule {
    pub message: String,
}

impl ApprovalTargetRule for ApprovalTargetRequiredLabelRule {
    fn check(&self, item: &ApprovalTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetRequiredCodeRule {
    pub message: String,
}

impl ApprovalTargetRule for ApprovalTargetRequiredCodeRule {
    fn check(&self, item: &ApprovalTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetCodePrefixRule {
    pub prefix: String,
}

impl ApprovalTargetRule for ApprovalTargetCodePrefixRule {
    fn check(&self, item: &ApprovalTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ApprovalTargetRule for ApprovalTargetExpiresAtRangeRule {
    fn check(&self, item: &ApprovalTarget) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a approval target.
#[derive(Debug, Clone)]
pub struct ApprovalTargetValidator {
    pub rules: Vec<Rc<dyn ApprovalTargetRule>>,
}

impl ApprovalTargetValidator {
    pub fn validate(&self, item: &ApprovalTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_target_validator() -> ApprovalTargetValidator {
    ApprovalTargetValidator {
        rules: vec![
            Rc::new(ApprovalTargetLabelLengthRule { max_length: 86400 }) as Rc<dyn ApprovalTargetRule>,
            Rc::new(ApprovalTargetRequiredAudienceRule {
                message: "approval target needs an entry".to_string(),
            }),
            Rc::new(ApprovalTargetRequiredLabelRule {
                message: "approval target needs this field".to_string(),
            }),
            Rc::new(ApprovalTargetRequiredCodeRule {
                message: "approval target needs this field".to_string(),
            }),
            Rc::new(ApprovalTargetCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ApprovalTargetExpiresAtRangeRule { min: 16, max: 1000 }),
        ],
    }
}
