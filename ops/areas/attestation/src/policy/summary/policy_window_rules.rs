use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy window must pass. Check returns its problems, or none.
pub trait PolicyWindowRule: std::fmt::Debug {
    fn check(&self, item: &PolicyWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowRequiredCodeRule {
    pub message: String,
}

impl PolicyWindowRule for PolicyWindowRequiredCodeRule {
    fn check(&self, item: &PolicyWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowCodePrefixRule {
    pub prefix: String,
}

impl PolicyWindowRule for PolicyWindowCodePrefixRule {
    fn check(&self, item: &PolicyWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyWindowRule for PolicyWindowEndsAtRangeRule {
    fn check(&self, item: &PolicyWindow) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyWindowRule for PolicyWindowCapacityRangeRule {
    fn check(&self, item: &PolicyWindow) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowScopeLengthRule {
    pub max_length: i64,
}

impl PolicyWindowRule for PolicyWindowScopeLengthRule {
    fn check(&self, item: &PolicyWindow) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PolicyWindowRule for PolicyWindowMinimumRangeRule {
    fn check(&self, item: &PolicyWindow) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a policy window.
#[derive(Debug, Clone)]
pub struct PolicyWindowValidator {
    pub rules: Vec<Rc<dyn PolicyWindowRule>>,
}

impl PolicyWindowValidator {
    pub fn validate(&self, item: &PolicyWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PolicyWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_window_validator() -> PolicyWindowValidator {
    PolicyWindowValidator {
        rules: vec![
            Rc::new(PolicyWindowRequiredCodeRule {
                message: "policy window needs an entry".to_string(),
            }) as Rc<dyn PolicyWindowRule>,
            Rc::new(PolicyWindowCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PolicyWindowEndsAtRangeRule { min: 32, max: 86400 }),
            Rc::new(PolicyWindowCapacityRangeRule { min: 16, max: 80 }),
            Rc::new(PolicyWindowScopeLengthRule { max_length: 86400 }),
            Rc::new(PolicyWindowMinimumRangeRule { min: 7.9, max: 4519.4 }),
        ],
    }
}
