use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit estimate must pass. Check returns its problems, or none.
pub trait LimitEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LimitEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimateCodeLengthRule {
    pub max_length: i64,
}

impl LimitEstimateRule for LimitEstimateCodeLengthRule {
    fn check(&self, item: &LimitEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimateDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitEstimateRule for LimitEstimateDiscountRangeRule {
    fn check(&self, item: &LimitEstimate) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimateCodePrefixRule {
    pub prefix: String,
}

impl LimitEstimateRule for LimitEstimateCodePrefixRule {
    fn check(&self, item: &LimitEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimateUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitEstimateRule for LimitEstimateUpdatedAtRangeRule {
    fn check(&self, item: &LimitEstimate) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimateRequiredCodeRule {
    pub message: String,
}

impl LimitEstimateRule for LimitEstimateRequiredCodeRule {
    fn check(&self, item: &LimitEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitEstimateNameLengthRule {
    pub max_length: i64,
}

impl LimitEstimateRule for LimitEstimateNameLengthRule {
    fn check(&self, item: &LimitEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit estimate.
#[derive(Debug, Clone)]
pub struct LimitEstimateValidator {
    pub rules: Vec<Rc<dyn LimitEstimateRule>>,
}

impl LimitEstimateValidator {
    pub fn validate(&self, item: &LimitEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_estimate_validator() -> LimitEstimateValidator {
    LimitEstimateValidator {
        rules: vec![
            Rc::new(LimitEstimateCodeLengthRule { max_length: 3600 }) as Rc<dyn LimitEstimateRule>,
            Rc::new(LimitEstimateDiscountRangeRule { min: 2.8, max: 1167.3 }),
            Rc::new(LimitEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(LimitEstimateUpdatedAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(LimitEstimateRequiredCodeRule {
                message: "limit estimate needs a value".to_string(),
            }),
            Rc::new(LimitEstimateNameLengthRule { max_length: 3600 }),
        ],
    }
}
