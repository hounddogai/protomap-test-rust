use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a release forecast must pass. Check returns its problems, or none.
pub trait ReleaseForecastRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseForecastAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl ReleaseForecastRule for ReleaseForecastAllowedReviewerRule {
    fn check(&self, item: &ReleaseForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is not allowed", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseForecastWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseForecastRule for ReleaseForecastWeightRangeRule {
    fn check(&self, item: &ReleaseForecast) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseForecastRequiredCodeRule {
    pub message: String,
}

impl ReleaseForecastRule for ReleaseForecastRequiredCodeRule {
    fn check(&self, item: &ReleaseForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseForecastRequiredTitleRule {
    pub message: String,
}

impl ReleaseForecastRule for ReleaseForecastRequiredTitleRule {
    fn check(&self, item: &ReleaseForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a release forecast.
#[derive(Debug, Clone)]
pub struct ReleaseForecastValidator {
    pub rules: Vec<Rc<dyn ReleaseForecastRule>>,
}

impl ReleaseForecastValidator {
    pub fn validate(&self, item: &ReleaseForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_forecast_validator() -> ReleaseForecastValidator {
    ReleaseForecastValidator {
        rules: vec![
            Rc::new(ReleaseForecastAllowedReviewerRule {
                allowed: vec!["legacy".to_string(), "tiered".to_string(), "tiered".to_string()],
            }) as Rc<dyn ReleaseForecastRule>,
            Rc::new(ReleaseForecastWeightRangeRule { min: 6.7, max: 594.9 }),
            Rc::new(ReleaseForecastRequiredCodeRule {
                message: "release forecast needs a value".to_string(),
            }),
            Rc::new(ReleaseForecastRequiredTitleRule {
                message: "release forecast needs an entry".to_string(),
            }),
        ],
    }
}
