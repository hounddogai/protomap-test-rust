use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback release exception must pass. Check returns its problems, or none.
pub trait FallbackReleaseExceptionRule: std::fmt::Debug {
    fn check(&self, item: &FallbackReleaseException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseExceptionTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackReleaseExceptionRule for FallbackReleaseExceptionTtlSecondsRangeRule {
    fn check(&self, item: &FallbackReleaseException) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseExceptionAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl FallbackReleaseExceptionRule for FallbackReleaseExceptionAllowedIssuerRule {
    fn check(&self, item: &FallbackReleaseException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is above maximum", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseExceptionRequiredIssuerRule {
    pub message: String,
}

impl FallbackReleaseExceptionRule for FallbackReleaseExceptionRequiredIssuerRule {
    fn check(&self, item: &FallbackReleaseException) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseExceptionGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackReleaseExceptionRule for FallbackReleaseExceptionGraceDaysRangeRule {
    fn check(&self, item: &FallbackReleaseException) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback release exception.
#[derive(Debug, Clone)]
pub struct FallbackReleaseExceptionValidator {
    pub rules: Vec<Rc<dyn FallbackReleaseExceptionRule>>,
}

impl FallbackReleaseExceptionValidator {
    pub fn validate(&self, item: &FallbackReleaseException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackReleaseException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackReleaseException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_release_exception_validator() -> FallbackReleaseExceptionValidator {
    FallbackReleaseExceptionValidator {
        rules: vec![
            Rc::new(FallbackReleaseExceptionTtlSecondsRangeRule { min: 16, max: 32 })
                as Rc<dyn FallbackReleaseExceptionRule>,
            Rc::new(FallbackReleaseExceptionAllowedIssuerRule {
                allowed: vec!["strict".to_string(), "lenient".to_string()],
            }),
            Rc::new(FallbackReleaseExceptionRequiredIssuerRule {
                message: "fallback release exception needs an entry".to_string(),
            }),
            Rc::new(FallbackReleaseExceptionGraceDaysRangeRule { min: 255, max: 1000 }),
        ],
    }
}
