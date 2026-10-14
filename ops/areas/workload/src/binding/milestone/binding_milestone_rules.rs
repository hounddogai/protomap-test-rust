use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding milestone must pass. Check returns its problems, or none.
pub trait BindingMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &BindingMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestoneRequiredTitleRule {
    pub message: String,
}

impl BindingMilestoneRule for BindingMilestoneRequiredTitleRule {
    fn check(&self, item: &BindingMilestone) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestoneRequiredCodeRule {
    pub message: String,
}

impl BindingMilestoneRule for BindingMilestoneRequiredCodeRule {
    fn check(&self, item: &BindingMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestoneCodePrefixRule {
    pub prefix: String,
}

impl BindingMilestoneRule for BindingMilestoneCodePrefixRule {
    fn check(&self, item: &BindingMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestoneGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BindingMilestoneRule for BindingMilestoneGraceDaysRangeRule {
    fn check(&self, item: &BindingMilestone) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a binding milestone.
#[derive(Debug, Clone)]
pub struct BindingMilestoneValidator {
    pub rules: Vec<Rc<dyn BindingMilestoneRule>>,
}

impl BindingMilestoneValidator {
    pub fn validate(&self, item: &BindingMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_milestone_validator() -> BindingMilestoneValidator {
    BindingMilestoneValidator {
        rules: vec![
            Rc::new(BindingMilestoneRequiredTitleRule {
                message: "binding milestone needs a value".to_string(),
            }) as Rc<dyn BindingMilestoneRule>,
            Rc::new(BindingMilestoneRequiredCodeRule {
                message: "binding milestone needs this field".to_string(),
            }),
            Rc::new(BindingMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(BindingMilestoneGraceDaysRangeRule { min: 64, max: 255 }),
        ],
    }
}
