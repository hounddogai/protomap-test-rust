use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist adjustment must pass. Check returns its problems, or none.
pub trait AllowlistAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAdjustmentSourceLengthRule {
    pub max_length: i64,
}

impl AllowlistAdjustmentRule for AllowlistAdjustmentSourceLengthRule {
    fn check(&self, item: &AllowlistAdjustment) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAdjustmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistAdjustmentRule for AllowlistAdjustmentAmountRangeRule {
    fn check(&self, item: &AllowlistAdjustment) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAdjustmentScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistAdjustmentRule for AllowlistAdjustmentScoreRangeRule {
    fn check(&self, item: &AllowlistAdjustment) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a allowlist adjustment.
#[derive(Debug, Clone)]
pub struct AllowlistAdjustmentValidator {
    pub rules: Vec<Rc<dyn AllowlistAdjustmentRule>>,
}

impl AllowlistAdjustmentValidator {
    pub fn validate(&self, item: &AllowlistAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_adjustment_validator() -> AllowlistAdjustmentValidator {
    AllowlistAdjustmentValidator {
        rules: vec![
            Rc::new(AllowlistAdjustmentSourceLengthRule { max_length: 255 }) as Rc<dyn AllowlistAdjustmentRule>,
            Rc::new(AllowlistAdjustmentAmountRangeRule { min: 10.0, max: 1716.4 }),
            Rc::new(AllowlistAdjustmentScoreRangeRule { min: 5.7, max: 2739.9 }),
        ],
    }
}
