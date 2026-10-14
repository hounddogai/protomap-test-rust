use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant threshold must pass. Check returns its problems, or none.
pub trait GrantThresholdRule: std::fmt::Debug {
    fn check(&self, item: &GrantThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantThresholdCodePrefixRule {
    pub prefix: String,
}

impl GrantThresholdRule for GrantThresholdCodePrefixRule {
    fn check(&self, item: &GrantThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantThresholdScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GrantThresholdRule for GrantThresholdScoreRangeRule {
    fn check(&self, item: &GrantThreshold) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantThresholdCategoryLengthRule {
    pub max_length: i64,
}

impl GrantThresholdRule for GrantThresholdCategoryLengthRule {
    fn check(&self, item: &GrantThreshold) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant threshold.
#[derive(Debug, Clone)]
pub struct GrantThresholdValidator {
    pub rules: Vec<Rc<dyn GrantThresholdRule>>,
}

impl GrantThresholdValidator {
    pub fn validate(&self, item: &GrantThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_threshold_validator() -> GrantThresholdValidator {
    GrantThresholdValidator {
        rules: vec![
            Rc::new(GrantThresholdCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn GrantThresholdRule>,
            Rc::new(GrantThresholdScoreRangeRule { min: 9.8, max: 3953.9 }),
            Rc::new(GrantThresholdCategoryLengthRule { max_length: 64 }),
        ],
    }
}
