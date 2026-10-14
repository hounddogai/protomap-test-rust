use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched policy forecast must pass. Check returns its problems, or none.
pub trait BatchedPolicyForecastRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecastRequiredReviewerRule {
    pub message: String,
}

impl BatchedPolicyForecastRule for BatchedPolicyForecastRequiredReviewerRule {
    fn check(&self, item: &BatchedPolicyForecast) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedPolicyForecastRule for BatchedPolicyForecastAllowedCodeRule {
    fn check(&self, item: &BatchedPolicyForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecastTitleLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyForecastRule for BatchedPolicyForecastTitleLengthRule {
    fn check(&self, item: &BatchedPolicyForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecastRequiredCodeRule {
    pub message: String,
}

impl BatchedPolicyForecastRule for BatchedPolicyForecastRequiredCodeRule {
    fn check(&self, item: &BatchedPolicyForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyForecastCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedPolicyForecastRule for BatchedPolicyForecastCeilingRangeRule {
    fn check(&self, item: &BatchedPolicyForecast) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched policy forecast.
#[derive(Debug, Clone)]
pub struct BatchedPolicyForecastValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyForecastRule>>,
}

impl BatchedPolicyForecastValidator {
    pub fn validate(&self, item: &BatchedPolicyForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicyForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_forecast_validator() -> BatchedPolicyForecastValidator {
    BatchedPolicyForecastValidator {
        rules: vec![
            Rc::new(BatchedPolicyForecastRequiredReviewerRule {
                message: "batched policy forecast needs a value".to_string(),
            }) as Rc<dyn BatchedPolicyForecastRule>,
            Rc::new(BatchedPolicyForecastAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(BatchedPolicyForecastTitleLengthRule { max_length: 255 }),
            Rc::new(BatchedPolicyForecastRequiredCodeRule {
                message: "batched policy forecast needs this field".to_string(),
            }),
            Rc::new(BatchedPolicyForecastCeilingRangeRule { min: 2.6, max: 3779.0 }),
        ],
    }
}
