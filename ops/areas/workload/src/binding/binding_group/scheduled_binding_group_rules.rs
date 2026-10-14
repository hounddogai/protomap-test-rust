use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled binding group must pass. Check returns its problems, or none.
pub trait ScheduledBindingGroupRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledBindingGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingGroupLocaleLengthRule {
    pub max_length: i64,
}

impl ScheduledBindingGroupRule for ScheduledBindingGroupLocaleLengthRule {
    fn check(&self, item: &ScheduledBindingGroup) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingGroupRequiredCodeRule {
    pub message: String,
}

impl ScheduledBindingGroupRule for ScheduledBindingGroupRequiredCodeRule {
    fn check(&self, item: &ScheduledBindingGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingGroupPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledBindingGroupRule for ScheduledBindingGroupPortRangeRule {
    fn check(&self, item: &ScheduledBindingGroup) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled binding group.
#[derive(Debug, Clone)]
pub struct ScheduledBindingGroupValidator {
    pub rules: Vec<Rc<dyn ScheduledBindingGroupRule>>,
}

impl ScheduledBindingGroupValidator {
    pub fn validate(&self, item: &ScheduledBindingGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledBindingGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledBindingGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_binding_group_validator() -> ScheduledBindingGroupValidator {
    ScheduledBindingGroupValidator {
        rules: vec![
            Rc::new(ScheduledBindingGroupLocaleLengthRule { max_length: 120 }) as Rc<dyn ScheduledBindingGroupRule>,
            Rc::new(ScheduledBindingGroupRequiredCodeRule {
                message: "scheduled binding group needs this field".to_string(),
            }),
            Rc::new(ScheduledBindingGroupPortRangeRule { min: 120, max: 86400 }),
        ],
    }
}
