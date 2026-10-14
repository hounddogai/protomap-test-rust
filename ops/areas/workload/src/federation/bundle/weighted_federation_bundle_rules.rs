use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted federation bundle must pass. Check returns its problems, or none.
pub trait WeightedFederationBundleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedFederationBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleRequiredCodeRule {
    pub message: String,
}

impl WeightedFederationBundleRule for WeightedFederationBundleRequiredCodeRule {
    fn check(&self, item: &WeightedFederationBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleCodeLengthRule {
    pub max_length: i64,
}

impl WeightedFederationBundleRule for WeightedFederationBundleCodeLengthRule {
    fn check(&self, item: &WeightedFederationBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFederationBundleRule for WeightedFederationBundleUpdatedAtRangeRule {
    fn check(&self, item: &WeightedFederationBundle) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleSummaryLengthRule {
    pub max_length: i64,
}

impl WeightedFederationBundleRule for WeightedFederationBundleSummaryLengthRule {
    fn check(&self, item: &WeightedFederationBundle) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedFederationBundleRule for WeightedFederationBundleAttemptsRangeRule {
    fn check(&self, item: &WeightedFederationBundle) -> Vec<String> {
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

/// Runs every rule of a weighted federation bundle.
#[derive(Debug, Clone)]
pub struct WeightedFederationBundleValidator {
    pub rules: Vec<Rc<dyn WeightedFederationBundleRule>>,
}

impl WeightedFederationBundleValidator {
    pub fn validate(&self, item: &WeightedFederationBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedFederationBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedFederationBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_federation_bundle_validator() -> WeightedFederationBundleValidator {
    WeightedFederationBundleValidator {
        rules: vec![
            Rc::new(WeightedFederationBundleRequiredCodeRule {
                message: "weighted federation bundle needs an entry".to_string(),
            }) as Rc<dyn WeightedFederationBundleRule>,
            Rc::new(WeightedFederationBundleCodeLengthRule { max_length: 80 }),
            Rc::new(WeightedFederationBundleUpdatedAtRangeRule { min: 80, max: 80 }),
            Rc::new(WeightedFederationBundleSummaryLengthRule { max_length: 3600 }),
            Rc::new(WeightedFederationBundleAttemptsRangeRule { min: 64, max: 3600 }),
        ],
    }
}
