use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage window must pass. Check returns its problems, or none.
pub trait UsageWindowRule: std::fmt::Debug {
    fn check(&self, item: &UsageWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindowCodeLengthRule {
    pub max_length: i64,
}

impl UsageWindowRule for UsageWindowCodeLengthRule {
    fn check(&self, item: &UsageWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindowCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageWindowRule for UsageWindowCeilingRangeRule {
    fn check(&self, item: &UsageWindow) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindowScopeLengthRule {
    pub max_length: i64,
}

impl UsageWindowRule for UsageWindowScopeLengthRule {
    fn check(&self, item: &UsageWindow) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindowRequiredCodeRule {
    pub message: String,
}

impl UsageWindowRule for UsageWindowRequiredCodeRule {
    fn check(&self, item: &UsageWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindowRequiredRegionRule {
    pub message: String,
}

impl UsageWindowRule for UsageWindowRequiredRegionRule {
    fn check(&self, item: &UsageWindow) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageWindowRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl UsageWindowRule for UsageWindowRatioRangeRule {
    fn check(&self, item: &UsageWindow) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a usage window.
#[derive(Debug, Clone)]
pub struct UsageWindowValidator {
    pub rules: Vec<Rc<dyn UsageWindowRule>>,
}

impl UsageWindowValidator {
    pub fn validate(&self, item: &UsageWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_window_validator() -> UsageWindowValidator {
    UsageWindowValidator {
        rules: vec![
            Rc::new(UsageWindowCodeLengthRule { max_length: 80 }) as Rc<dyn UsageWindowRule>,
            Rc::new(UsageWindowCeilingRangeRule { min: 4.6, max: 3510.3 }),
            Rc::new(UsageWindowScopeLengthRule { max_length: 120 }),
            Rc::new(UsageWindowRequiredCodeRule {
                message: "usage window needs this field".to_string(),
            }),
            Rc::new(UsageWindowRequiredRegionRule {
                message: "usage window needs this field".to_string(),
            }),
            Rc::new(UsageWindowRatioRangeRule { min: 7.7, max: 4342.4 }),
        ],
    }
}
