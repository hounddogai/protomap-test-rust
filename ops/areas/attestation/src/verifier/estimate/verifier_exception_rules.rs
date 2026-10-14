use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier exception must pass. Check returns its problems, or none.
pub trait VerifierExceptionRule: std::fmt::Debug {
    fn check(&self, item: &VerifierException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierExceptionAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl VerifierExceptionRule for VerifierExceptionAllowedTitleRule {
    fn check(&self, item: &VerifierException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierExceptionCodePrefixRule {
    pub prefix: String,
}

impl VerifierExceptionRule for VerifierExceptionCodePrefixRule {
    fn check(&self, item: &VerifierException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierExceptionThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierExceptionRule for VerifierExceptionThresholdRangeRule {
    fn check(&self, item: &VerifierException) -> Vec<String> {
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
pub struct VerifierExceptionCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierExceptionRule for VerifierExceptionCapacityRangeRule {
    fn check(&self, item: &VerifierException) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierExceptionTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl VerifierExceptionRule for VerifierExceptionTagsLimitRule {
    fn check(&self, item: &VerifierException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a verifier exception.
#[derive(Debug, Clone)]
pub struct VerifierExceptionValidator {
    pub rules: Vec<Rc<dyn VerifierExceptionRule>>,
}

impl VerifierExceptionValidator {
    pub fn validate(&self, item: &VerifierException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_exception_validator() -> VerifierExceptionValidator {
    VerifierExceptionValidator {
        rules: vec![
            Rc::new(VerifierExceptionAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn VerifierExceptionRule>,
            Rc::new(VerifierExceptionCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(VerifierExceptionThresholdRangeRule { min: 3.7, max: 4744.4 }),
            Rc::new(VerifierExceptionCapacityRangeRule { min: 80, max: 3600 }),
            Rc::new(VerifierExceptionTagsLimitRule {
                max_count: 86400,
                banned: vec!["weighted".to_string(), "global".to_string()],
            }),
        ],
    }
}
