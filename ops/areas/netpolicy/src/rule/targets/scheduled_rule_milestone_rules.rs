use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rule milestone must pass. Check returns its problems, or none.
pub trait ScheduledRuleMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRuleMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestoneStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRuleMilestoneRule for ScheduledRuleMilestoneStartsAtRangeRule {
    fn check(&self, item: &ScheduledRuleMilestone) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestoneCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRuleMilestoneRule for ScheduledRuleMilestoneCodePrefixRule {
    fn check(&self, item: &ScheduledRuleMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestoneCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledRuleMilestoneRule for ScheduledRuleMilestoneCodeLengthRule {
    fn check(&self, item: &ScheduledRuleMilestone) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestoneLocaleLengthRule {
    pub max_length: i64,
}

impl ScheduledRuleMilestoneRule for ScheduledRuleMilestoneLocaleLengthRule {
    fn check(&self, item: &ScheduledRuleMilestone) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleMilestoneRequiredLocaleRule {
    pub message: String,
}

impl ScheduledRuleMilestoneRule for ScheduledRuleMilestoneRequiredLocaleRule {
    fn check(&self, item: &ScheduledRuleMilestone) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled rule milestone.
#[derive(Debug, Clone)]
pub struct ScheduledRuleMilestoneValidator {
    pub rules: Vec<Rc<dyn ScheduledRuleMilestoneRule>>,
}

impl ScheduledRuleMilestoneValidator {
    pub fn validate(&self, item: &ScheduledRuleMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRuleMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRuleMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rule_milestone_validator() -> ScheduledRuleMilestoneValidator {
    ScheduledRuleMilestoneValidator {
        rules: vec![
            Rc::new(ScheduledRuleMilestoneStartsAtRangeRule { min: 3600, max: 86400 })
                as Rc<dyn ScheduledRuleMilestoneRule>,
            Rc::new(ScheduledRuleMilestoneCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(ScheduledRuleMilestoneCodeLengthRule { max_length: 32 }),
            Rc::new(ScheduledRuleMilestoneLocaleLengthRule { max_length: 32 }),
            Rc::new(ScheduledRuleMilestoneRequiredLocaleRule {
                message: "scheduled rule milestone needs this field".to_string(),
            }),
        ],
    }
}
