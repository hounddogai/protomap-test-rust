use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier bundle must pass. Check returns its problems, or none.
pub trait VerifierBundleRule: std::fmt::Debug {
    fn check(&self, item: &VerifierBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierBundleRule for VerifierBundleMinimumRangeRule {
    fn check(&self, item: &VerifierBundle) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleSourceLengthRule {
    pub max_length: i64,
}

impl VerifierBundleRule for VerifierBundleSourceLengthRule {
    fn check(&self, item: &VerifierBundle) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierBundleRule for VerifierBundleCreatedAtRangeRule {
    fn check(&self, item: &VerifierBundle) -> Vec<String> {
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
pub struct VerifierBundleCodeLengthRule {
    pub max_length: i64,
}

impl VerifierBundleRule for VerifierBundleCodeLengthRule {
    fn check(&self, item: &VerifierBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleRequiredLabelRule {
    pub message: String,
}

impl VerifierBundleRule for VerifierBundleRequiredLabelRule {
    fn check(&self, item: &VerifierBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBundleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierBundleRule for VerifierBundleSequenceRangeRule {
    fn check(&self, item: &VerifierBundle) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier bundle.
#[derive(Debug, Clone)]
pub struct VerifierBundleValidator {
    pub rules: Vec<Rc<dyn VerifierBundleRule>>,
}

impl VerifierBundleValidator {
    pub fn validate(&self, item: &VerifierBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_bundle_validator() -> VerifierBundleValidator {
    VerifierBundleValidator {
        rules: vec![
            Rc::new(VerifierBundleMinimumRangeRule { min: 5.8, max: 4470.4 }) as Rc<dyn VerifierBundleRule>,
            Rc::new(VerifierBundleSourceLengthRule { max_length: 64 }),
            Rc::new(VerifierBundleCreatedAtRangeRule { min: 64, max: 1000 }),
            Rc::new(VerifierBundleCodeLengthRule { max_length: 64 }),
            Rc::new(VerifierBundleRequiredLabelRule {
                message: "verifier bundle needs a value".to_string(),
            }),
            Rc::new(VerifierBundleSequenceRangeRule { min: 255, max: 255 }),
        ],
    }
}
