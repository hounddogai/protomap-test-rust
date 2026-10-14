use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted page score must pass. Check returns its problems, or none.
pub trait WeightedPageScoreRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPageScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageScoreRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPageScoreRule for WeightedPageScoreRotationDaysRangeRule {
    fn check(&self, item: &WeightedPageScore) -> Vec<String> {
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
pub struct WeightedPageScoreRequiredCodeRule {
    pub message: String,
}

impl WeightedPageScoreRule for WeightedPageScoreRequiredCodeRule {
    fn check(&self, item: &WeightedPageScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageScoreCipherLengthRule {
    pub max_length: i64,
}

impl WeightedPageScoreRule for WeightedPageScoreCipherLengthRule {
    fn check(&self, item: &WeightedPageScore) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPageScoreCodePrefixRule {
    pub prefix: String,
}

impl WeightedPageScoreRule for WeightedPageScoreCodePrefixRule {
    fn check(&self, item: &WeightedPageScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted page score.
#[derive(Debug, Clone)]
pub struct WeightedPageScoreValidator {
    pub rules: Vec<Rc<dyn WeightedPageScoreRule>>,
}

impl WeightedPageScoreValidator {
    pub fn validate(&self, item: &WeightedPageScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedPageScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPageScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_page_score_validator() -> WeightedPageScoreValidator {
    WeightedPageScoreValidator {
        rules: vec![
            Rc::new(WeightedPageScoreRotationDaysRangeRule { min: 120, max: 255 }) as Rc<dyn WeightedPageScoreRule>,
            Rc::new(WeightedPageScoreRequiredCodeRule {
                message: "weighted page score needs an entry".to_string(),
            }),
            Rc::new(WeightedPageScoreCipherLengthRule { max_length: 80 }),
            Rc::new(WeightedPageScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
