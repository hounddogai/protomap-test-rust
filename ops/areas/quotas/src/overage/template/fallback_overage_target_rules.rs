use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback overage target must pass. Check returns its problems, or none.
pub trait FallbackOverageTargetRule: std::fmt::Debug {
    fn check(&self, item: &FallbackOverageTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackOverageTargetRule for FallbackOverageTargetGraceDaysRangeRule {
    fn check(&self, item: &FallbackOverageTarget) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetRegionLengthRule {
    pub max_length: i64,
}

impl FallbackOverageTargetRule for FallbackOverageTargetRegionLengthRule {
    fn check(&self, item: &FallbackOverageTarget) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl FallbackOverageTargetRule for FallbackOverageTargetAllowedSourceRule {
    fn check(&self, item: &FallbackOverageTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is too long", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTargetRequiredNameRule {
    pub message: String,
}

impl FallbackOverageTargetRule for FallbackOverageTargetRequiredNameRule {
    fn check(&self, item: &FallbackOverageTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback overage target.
#[derive(Debug, Clone)]
pub struct FallbackOverageTargetValidator {
    pub rules: Vec<Rc<dyn FallbackOverageTargetRule>>,
}

impl FallbackOverageTargetValidator {
    pub fn validate(&self, item: &FallbackOverageTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackOverageTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackOverageTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_overage_target_validator() -> FallbackOverageTargetValidator {
    FallbackOverageTargetValidator {
        rules: vec![
            Rc::new(FallbackOverageTargetGraceDaysRangeRule { min: 80, max: 3600 })
                as Rc<dyn FallbackOverageTargetRule>,
            Rc::new(FallbackOverageTargetRegionLengthRule { max_length: 32 }),
            Rc::new(FallbackOverageTargetAllowedSourceRule {
                allowed: vec!["primary".to_string()],
            }),
            Rc::new(FallbackOverageTargetRequiredNameRule {
                message: "fallback overage target needs a value".to_string(),
            }),
        ],
    }
}
