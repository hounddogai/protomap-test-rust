use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a grant adjustment must pass. Check returns its problems, or none.
pub trait GrantAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &GrantAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantAdjustmentRule for GrantAdjustmentThresholdRangeRule {
    fn check(&self, item: &GrantAdjustment) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GrantAdjustmentRule for GrantAdjustmentAllowedCodeRule {
    fn check(&self, item: &GrantAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl GrantAdjustmentRule for GrantAdjustmentTitleLengthRule {
    fn check(&self, item: &GrantAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GrantAdjustmentRule for GrantAdjustmentRetriesRangeRule {
    fn check(&self, item: &GrantAdjustment) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantAdjustmentRule for GrantAdjustmentRateRangeRule {
    fn check(&self, item: &GrantAdjustment) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantAdjustmentRequiredKeyIdRule {
    pub message: String,
}

impl GrantAdjustmentRule for GrantAdjustmentRequiredKeyIdRule {
    fn check(&self, item: &GrantAdjustment) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant adjustment.
#[derive(Debug, Clone)]
pub struct GrantAdjustmentValidator {
    pub rules: Vec<Rc<dyn GrantAdjustmentRule>>,
}

impl GrantAdjustmentValidator {
    pub fn validate(&self, item: &GrantAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_adjustment_validator() -> GrantAdjustmentValidator {
    GrantAdjustmentValidator {
        rules: vec![
            Rc::new(GrantAdjustmentThresholdRangeRule { min: 7.4, max: 3952.6 }) as Rc<dyn GrantAdjustmentRule>,
            Rc::new(GrantAdjustmentAllowedCodeRule {
                allowed: vec!["global".to_string(), "default".to_string()],
            }),
            Rc::new(GrantAdjustmentTitleLengthRule { max_length: 1000 }),
            Rc::new(GrantAdjustmentRetriesRangeRule { min: 16, max: 64 }),
            Rc::new(GrantAdjustmentRateRangeRule { min: 2.9, max: 924.4 }),
            Rc::new(GrantAdjustmentRequiredKeyIdRule {
                message: "grant adjustment needs this field".to_string(),
            }),
        ],
    }
}
