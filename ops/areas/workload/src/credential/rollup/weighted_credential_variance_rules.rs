use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted credential variance must pass. Check returns its problems, or none.
pub trait WeightedCredentialVarianceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialVarianceKeyIdLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialVarianceRule for WeightedCredentialVarianceKeyIdLengthRule {
    fn check(&self, item: &WeightedCredentialVariance) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialVarianceLabelLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialVarianceRule for WeightedCredentialVarianceLabelLengthRule {
    fn check(&self, item: &WeightedCredentialVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialVarianceMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedCredentialVarianceRule for WeightedCredentialVarianceMinimumRangeRule {
    fn check(&self, item: &WeightedCredentialVariance) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialVarianceAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl WeightedCredentialVarianceRule for WeightedCredentialVarianceAllowedCategoryRule {
    fn check(&self, item: &WeightedCredentialVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is stale", item.category)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential variance.
#[derive(Debug, Clone)]
pub struct WeightedCredentialVarianceValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialVarianceRule>>,
}

impl WeightedCredentialVarianceValidator {
    pub fn validate(&self, item: &WeightedCredentialVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedCredentialVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_variance_validator() -> WeightedCredentialVarianceValidator {
    WeightedCredentialVarianceValidator {
        rules: vec![
            Rc::new(WeightedCredentialVarianceKeyIdLengthRule { max_length: 86400 })
                as Rc<dyn WeightedCredentialVarianceRule>,
            Rc::new(WeightedCredentialVarianceLabelLengthRule { max_length: 80 }),
            Rc::new(WeightedCredentialVarianceMinimumRangeRule { min: 8.5, max: 1933.3 }),
            Rc::new(WeightedCredentialVarianceAllowedCategoryRule { allowed: Vec::new() }),
        ],
    }
}
