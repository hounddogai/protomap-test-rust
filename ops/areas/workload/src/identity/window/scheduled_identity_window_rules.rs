use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled identity window must pass. Check returns its problems, or none.
pub trait ScheduledIdentityWindowRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledIdentityWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindowCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledIdentityWindowRule for ScheduledIdentityWindowCodeLengthRule {
    fn check(&self, item: &ScheduledIdentityWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindowScopeLengthRule {
    pub max_length: i64,
}

impl ScheduledIdentityWindowRule for ScheduledIdentityWindowScopeLengthRule {
    fn check(&self, item: &ScheduledIdentityWindow) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindowTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledIdentityWindowRule for ScheduledIdentityWindowTitleLengthRule {
    fn check(&self, item: &ScheduledIdentityWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindowQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledIdentityWindowRule for ScheduledIdentityWindowQuantityRangeRule {
    fn check(&self, item: &ScheduledIdentityWindow) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindowRequiredScopeRule {
    pub message: String,
}

impl ScheduledIdentityWindowRule for ScheduledIdentityWindowRequiredScopeRule {
    fn check(&self, item: &ScheduledIdentityWindow) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled identity window.
#[derive(Debug, Clone)]
pub struct ScheduledIdentityWindowValidator {
    pub rules: Vec<Rc<dyn ScheduledIdentityWindowRule>>,
}

impl ScheduledIdentityWindowValidator {
    pub fn validate(&self, item: &ScheduledIdentityWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledIdentityWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledIdentityWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_identity_window_validator() -> ScheduledIdentityWindowValidator {
    ScheduledIdentityWindowValidator {
        rules: vec![
            Rc::new(ScheduledIdentityWindowCodeLengthRule { max_length: 120 }) as Rc<dyn ScheduledIdentityWindowRule>,
            Rc::new(ScheduledIdentityWindowScopeLengthRule { max_length: 255 }),
            Rc::new(ScheduledIdentityWindowTitleLengthRule { max_length: 86400 }),
            Rc::new(ScheduledIdentityWindowQuantityRangeRule { min: 64, max: 64 }),
            Rc::new(ScheduledIdentityWindowRequiredScopeRule {
                message: "scheduled identity window needs this field".to_string(),
            }),
        ],
    }
}
