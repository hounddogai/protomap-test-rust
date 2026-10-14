use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted query profile must pass. Check returns its problems, or none.
pub trait WeightedQueryProfileRule: std::fmt::Debug {
    fn check(&self, item: &WeightedQueryProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryProfileAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedQueryProfileRule for WeightedQueryProfileAttemptsRangeRule {
    fn check(&self, item: &WeightedQueryProfile) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryProfileRequiredHostRule {
    pub message: String,
}

impl WeightedQueryProfileRule for WeightedQueryProfileRequiredHostRule {
    fn check(&self, item: &WeightedQueryProfile) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryProfileCodePrefixRule {
    pub prefix: String,
}

impl WeightedQueryProfileRule for WeightedQueryProfileCodePrefixRule {
    fn check(&self, item: &WeightedQueryProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryProfileTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedQueryProfileRule for WeightedQueryProfileTtlSecondsRangeRule {
    fn check(&self, item: &WeightedQueryProfile) -> Vec<String> {
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
pub struct WeightedQueryProfileCodeLengthRule {
    pub max_length: i64,
}

impl WeightedQueryProfileRule for WeightedQueryProfileCodeLengthRule {
    fn check(&self, item: &WeightedQueryProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted query profile.
#[derive(Debug, Clone)]
pub struct WeightedQueryProfileValidator {
    pub rules: Vec<Rc<dyn WeightedQueryProfileRule>>,
}

impl WeightedQueryProfileValidator {
    pub fn validate(&self, item: &WeightedQueryProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedQueryProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedQueryProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_query_profile_validator() -> WeightedQueryProfileValidator {
    WeightedQueryProfileValidator {
        rules: vec![
            Rc::new(WeightedQueryProfileAttemptsRangeRule { min: 86400, max: 86400 })
                as Rc<dyn WeightedQueryProfileRule>,
            Rc::new(WeightedQueryProfileRequiredHostRule {
                message: "weighted query profile needs an entry".to_string(),
            }),
            Rc::new(WeightedQueryProfileCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(WeightedQueryProfileTtlSecondsRangeRule { min: 80, max: 255 }),
            Rc::new(WeightedQueryProfileCodeLengthRule { max_length: 3600 }),
        ],
    }
}
