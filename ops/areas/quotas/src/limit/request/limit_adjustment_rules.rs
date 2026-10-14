use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit adjustment must pass. Check returns its problems, or none.
pub trait LimitAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &LimitAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LimitAdjustmentRule for LimitAdjustmentAllowedCodeRule {
    fn check(&self, item: &LimitAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl LimitAdjustmentRule for LimitAdjustmentLabelLengthRule {
    fn check(&self, item: &LimitAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAdjustmentRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitAdjustmentRule for LimitAdjustmentRateRangeRule {
    fn check(&self, item: &LimitAdjustment) -> Vec<String> {
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
pub struct LimitAdjustmentActiveGuardRule {
    pub limit: f64,
}

impl LimitAdjustmentRule for LimitAdjustmentActiveGuardRule {
    fn check(&self, item: &LimitAdjustment) -> Vec<String> {
        if item.active && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while active", item.margin, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAdjustmentRequiredLabelRule {
    pub message: String,
}

impl LimitAdjustmentRule for LimitAdjustmentRequiredLabelRule {
    fn check(&self, item: &LimitAdjustment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAdjustmentMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitAdjustmentRule for LimitAdjustmentMarginRangeRule {
    fn check(&self, item: &LimitAdjustment) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit adjustment.
#[derive(Debug, Clone)]
pub struct LimitAdjustmentValidator {
    pub rules: Vec<Rc<dyn LimitAdjustmentRule>>,
}

impl LimitAdjustmentValidator {
    pub fn validate(&self, item: &LimitAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_adjustment_validator() -> LimitAdjustmentValidator {
    LimitAdjustmentValidator {
        rules: vec![
            Rc::new(LimitAdjustmentAllowedCodeRule {
                allowed: vec!["regional".to_string()],
            }) as Rc<dyn LimitAdjustmentRule>,
            Rc::new(LimitAdjustmentLabelLengthRule { max_length: 255 }),
            Rc::new(LimitAdjustmentRateRangeRule { min: 3.9, max: 2791.8 }),
            Rc::new(LimitAdjustmentActiveGuardRule { limit: 8.3 }),
            Rc::new(LimitAdjustmentRequiredLabelRule {
                message: "limit adjustment needs a value".to_string(),
            }),
            Rc::new(LimitAdjustmentMarginRangeRule { min: 4.1, max: 735.6 }),
        ],
    }
}
