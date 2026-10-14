use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal identity cycle must pass. Check returns its problems, or none.
pub trait SeasonalIdentityCycleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalIdentityCycleRule for SeasonalIdentityCycleRotationDaysRangeRule {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycleRequiredCodeRule {
    pub message: String,
}

impl SeasonalIdentityCycleRule for SeasonalIdentityCycleRequiredCodeRule {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalIdentityCycleRule for SeasonalIdentityCycleAttemptsRangeRule {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCyclePinnedGuardRule {
    pub limit: i64,
}

impl SeasonalIdentityCycleRule for SeasonalIdentityCyclePinnedGuardRule {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        if item.pinned && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while pinned",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycleRequiredSourceRule {
    pub message: String,
}

impl SeasonalIdentityCycleRule for SeasonalIdentityCycleRequiredSourceRule {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycleSourceLengthRule {
    pub max_length: i64,
}

impl SeasonalIdentityCycleRule for SeasonalIdentityCycleSourceLengthRule {
    fn check(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal identity cycle.
#[derive(Debug, Clone)]
pub struct SeasonalIdentityCycleValidator {
    pub rules: Vec<Rc<dyn SeasonalIdentityCycleRule>>,
}

impl SeasonalIdentityCycleValidator {
    pub fn validate(&self, item: &SeasonalIdentityCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalIdentityCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalIdentityCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_identity_cycle_validator() -> SeasonalIdentityCycleValidator {
    SeasonalIdentityCycleValidator {
        rules: vec![
            Rc::new(SeasonalIdentityCycleRotationDaysRangeRule { min: 255, max: 86400 })
                as Rc<dyn SeasonalIdentityCycleRule>,
            Rc::new(SeasonalIdentityCycleRequiredCodeRule {
                message: "seasonal identity cycle needs this field".to_string(),
            }),
            Rc::new(SeasonalIdentityCycleAttemptsRangeRule { min: 80, max: 86400 }),
            Rc::new(SeasonalIdentityCyclePinnedGuardRule { limit: 1000 }),
            Rc::new(SeasonalIdentityCycleRequiredSourceRule {
                message: "seasonal identity cycle needs an entry".to_string(),
            }),
            Rc::new(SeasonalIdentityCycleSourceLengthRule { max_length: 86400 }),
        ],
    }
}
