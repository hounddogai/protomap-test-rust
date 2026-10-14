use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy binding claim must pass. Check returns its problems, or none.
pub trait LegacyBindingClaimRule: std::fmt::Debug {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyBindingClaimRule for LegacyBindingClaimGraceDaysRangeRule {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String> {
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
pub struct LegacyBindingClaimRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyBindingClaimRule for LegacyBindingClaimRetriesRangeRule {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimRequiredReviewerRule {
    pub message: String,
}

impl LegacyBindingClaimRule for LegacyBindingClaimRequiredReviewerRule {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyBindingClaimRule for LegacyBindingClaimCeilingRangeRule {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimReviewerLengthRule {
    pub max_length: i64,
}

impl LegacyBindingClaimRule for LegacyBindingClaimReviewerLengthRule {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaimNameLengthRule {
    pub max_length: i64,
}

impl LegacyBindingClaimRule for LegacyBindingClaimNameLengthRule {
    fn check(&self, item: &LegacyBindingClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy binding claim.
#[derive(Debug, Clone)]
pub struct LegacyBindingClaimValidator {
    pub rules: Vec<Rc<dyn LegacyBindingClaimRule>>,
}

impl LegacyBindingClaimValidator {
    pub fn validate(&self, item: &LegacyBindingClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyBindingClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyBindingClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_binding_claim_validator() -> LegacyBindingClaimValidator {
    LegacyBindingClaimValidator {
        rules: vec![
            Rc::new(LegacyBindingClaimGraceDaysRangeRule { min: 80, max: 3600 }) as Rc<dyn LegacyBindingClaimRule>,
            Rc::new(LegacyBindingClaimRetriesRangeRule { min: 32, max: 120 }),
            Rc::new(LegacyBindingClaimRequiredReviewerRule {
                message: "legacy binding claim needs a value".to_string(),
            }),
            Rc::new(LegacyBindingClaimCeilingRangeRule { min: 8.6, max: 4302.6 }),
            Rc::new(LegacyBindingClaimReviewerLengthRule { max_length: 64 }),
            Rc::new(LegacyBindingClaimNameLengthRule { max_length: 255 }),
        ],
    }
}
