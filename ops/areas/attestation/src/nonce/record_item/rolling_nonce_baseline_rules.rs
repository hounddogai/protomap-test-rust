use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling nonce baseline must pass. Check returns its problems, or none.
pub trait RollingNonceBaselineRule: std::fmt::Debug {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineRequiredKeyIdRule {
    pub message: String,
}

impl RollingNonceBaselineRule for RollingNonceBaselineRequiredKeyIdRule {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingNonceBaselineRule for RollingNonceBaselineCreatedAtRangeRule {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineRequiredChecksumRule {
    pub message: String,
}

impl RollingNonceBaselineRule for RollingNonceBaselineRequiredChecksumRule {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineTitleLengthRule {
    pub max_length: i64,
}

impl RollingNonceBaselineRule for RollingNonceBaselineTitleLengthRule {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingNonceBaselineRule for RollingNonceBaselineDiscountRangeRule {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBaselineRequiredTitleRule {
    pub message: String,
}

impl RollingNonceBaselineRule for RollingNonceBaselineRequiredTitleRule {
    fn check(&self, item: &RollingNonceBaseline) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling nonce baseline.
#[derive(Debug, Clone)]
pub struct RollingNonceBaselineValidator {
    pub rules: Vec<Rc<dyn RollingNonceBaselineRule>>,
}

impl RollingNonceBaselineValidator {
    pub fn validate(&self, item: &RollingNonceBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingNonceBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingNonceBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_nonce_baseline_validator() -> RollingNonceBaselineValidator {
    RollingNonceBaselineValidator {
        rules: vec![
            Rc::new(RollingNonceBaselineRequiredKeyIdRule {
                message: "rolling nonce baseline needs an entry".to_string(),
            }) as Rc<dyn RollingNonceBaselineRule>,
            Rc::new(RollingNonceBaselineCreatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(RollingNonceBaselineRequiredChecksumRule {
                message: "rolling nonce baseline needs this field".to_string(),
            }),
            Rc::new(RollingNonceBaselineTitleLengthRule { max_length: 16 }),
            Rc::new(RollingNonceBaselineDiscountRangeRule { min: 7.8, max: 1560.6 }),
            Rc::new(RollingNonceBaselineRequiredTitleRule {
                message: "rolling nonce baseline needs an entry".to_string(),
            }),
        ],
    }
}
