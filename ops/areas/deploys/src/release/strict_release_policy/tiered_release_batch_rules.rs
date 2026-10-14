use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered release batch must pass. Check returns its problems, or none.
pub trait TieredReleaseBatchRule: std::fmt::Debug {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBatchAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl TieredReleaseBatchRule for TieredReleaseBatchAllowedReviewerRule {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is missing", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBatchCodePrefixRule {
    pub prefix: String,
}

impl TieredReleaseBatchRule for TieredReleaseBatchCodePrefixRule {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBatchNameLengthRule {
    pub max_length: i64,
}

impl TieredReleaseBatchRule for TieredReleaseBatchNameLengthRule {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBatchReviewerLengthRule {
    pub max_length: i64,
}

impl TieredReleaseBatchRule for TieredReleaseBatchReviewerLengthRule {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String> {
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
pub struct TieredReleaseBatchCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredReleaseBatchRule for TieredReleaseBatchCeilingRangeRule {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBatchRequiredReviewerRule {
    pub message: String,
}

impl TieredReleaseBatchRule for TieredReleaseBatchRequiredReviewerRule {
    fn check(&self, item: &TieredReleaseBatch) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered release batch.
#[derive(Debug, Clone)]
pub struct TieredReleaseBatchValidator {
    pub rules: Vec<Rc<dyn TieredReleaseBatchRule>>,
}

impl TieredReleaseBatchValidator {
    pub fn validate(&self, item: &TieredReleaseBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredReleaseBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredReleaseBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_release_batch_validator() -> TieredReleaseBatchValidator {
    TieredReleaseBatchValidator {
        rules: vec![
            Rc::new(TieredReleaseBatchAllowedReviewerRule {
                allowed: vec!["default".to_string(), "fallback".to_string()],
            }) as Rc<dyn TieredReleaseBatchRule>,
            Rc::new(TieredReleaseBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TieredReleaseBatchNameLengthRule { max_length: 80 }),
            Rc::new(TieredReleaseBatchReviewerLengthRule { max_length: 32 }),
            Rc::new(TieredReleaseBatchCeilingRangeRule { min: 1.9, max: 4019.5 }),
            Rc::new(TieredReleaseBatchRequiredReviewerRule {
                message: "tiered release batch needs a value".to_string(),
            }),
        ],
    }
}
