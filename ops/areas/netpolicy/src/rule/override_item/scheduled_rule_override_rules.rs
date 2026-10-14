use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled rule override must pass. Check returns its problems, or none.
pub trait ScheduledRuleOverrideRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledRuleOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledRuleOverrideRule for ScheduledRuleOverrideTitleLengthRule {
    fn check(&self, item: &ScheduledRuleOverride) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideRequiredTitleRule {
    pub message: String,
}

impl ScheduledRuleOverrideRule for ScheduledRuleOverrideRequiredTitleRule {
    fn check(&self, item: &ScheduledRuleOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledRuleOverrideRule for ScheduledRuleOverrideAmountRangeRule {
    fn check(&self, item: &ScheduledRuleOverride) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideCodePrefixRule {
    pub prefix: String,
}

impl ScheduledRuleOverrideRule for ScheduledRuleOverrideCodePrefixRule {
    fn check(&self, item: &ScheduledRuleOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverrideGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledRuleOverrideRule for ScheduledRuleOverrideGraceDaysRangeRule {
    fn check(&self, item: &ScheduledRuleOverride) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled rule override.
#[derive(Debug, Clone)]
pub struct ScheduledRuleOverrideValidator {
    pub rules: Vec<Rc<dyn ScheduledRuleOverrideRule>>,
}

impl ScheduledRuleOverrideValidator {
    pub fn validate(&self, item: &ScheduledRuleOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledRuleOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledRuleOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_rule_override_validator() -> ScheduledRuleOverrideValidator {
    ScheduledRuleOverrideValidator {
        rules: vec![
            Rc::new(ScheduledRuleOverrideTitleLengthRule { max_length: 86400 }) as Rc<dyn ScheduledRuleOverrideRule>,
            Rc::new(ScheduledRuleOverrideRequiredTitleRule {
                message: "scheduled rule override needs this field".to_string(),
            }),
            Rc::new(ScheduledRuleOverrideAmountRangeRule { min: 6.2, max: 2092.8 }),
            Rc::new(ScheduledRuleOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ScheduledRuleOverrideGraceDaysRangeRule { min: 120, max: 1000 }),
        ],
    }
}
