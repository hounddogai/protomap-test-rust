use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal grant window must pass. Check returns its problems, or none.
pub trait SeasonalGrantWindowRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowRequiredCodeRule {
    pub message: String,
}

impl SeasonalGrantWindowRule for SeasonalGrantWindowRequiredCodeRule {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalGrantWindowRule for SeasonalGrantWindowPriorityRangeRule {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalGrantWindowRule for SeasonalGrantWindowBatchSizeRangeRule {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowCodePrefixRule {
    pub prefix: String,
}

impl SeasonalGrantWindowRule for SeasonalGrantWindowCodePrefixRule {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalGrantWindowRule for SeasonalGrantWindowTtlSecondsRangeRule {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalGrantWindowRequiredCategoryRule {
    pub message: String,
}

impl SeasonalGrantWindowRule for SeasonalGrantWindowRequiredCategoryRule {
    fn check(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal grant window.
#[derive(Debug, Clone)]
pub struct SeasonalGrantWindowValidator {
    pub rules: Vec<Rc<dyn SeasonalGrantWindowRule>>,
}

impl SeasonalGrantWindowValidator {
    pub fn validate(&self, item: &SeasonalGrantWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalGrantWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalGrantWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_grant_window_validator() -> SeasonalGrantWindowValidator {
    SeasonalGrantWindowValidator {
        rules: vec![
            Rc::new(SeasonalGrantWindowRequiredCodeRule {
                message: "seasonal grant window needs a value".to_string(),
            }) as Rc<dyn SeasonalGrantWindowRule>,
            Rc::new(SeasonalGrantWindowPriorityRangeRule { min: 80, max: 86400 }),
            Rc::new(SeasonalGrantWindowBatchSizeRangeRule { min: 32, max: 80 }),
            Rc::new(SeasonalGrantWindowCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalGrantWindowTtlSecondsRangeRule { min: 16, max: 86400 }),
            Rc::new(SeasonalGrantWindowRequiredCategoryRule {
                message: "seasonal grant window needs an entry".to_string(),
            }),
        ],
    }
}
