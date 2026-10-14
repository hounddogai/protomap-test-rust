use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched federation forecast must pass. Check returns its problems, or none.
pub trait BatchedFederationForecastRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFederationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationForecastLocaleLengthRule {
    pub max_length: i64,
}

impl BatchedFederationForecastRule for BatchedFederationForecastLocaleLengthRule {
    fn check(&self, item: &BatchedFederationForecast) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationForecastRegionLengthRule {
    pub max_length: i64,
}

impl BatchedFederationForecastRule for BatchedFederationForecastRegionLengthRule {
    fn check(&self, item: &BatchedFederationForecast) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationForecastRequiredLocaleRule {
    pub message: String,
}

impl BatchedFederationForecastRule for BatchedFederationForecastRequiredLocaleRule {
    fn check(&self, item: &BatchedFederationForecast) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationForecastThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFederationForecastRule for BatchedFederationForecastThresholdRangeRule {
    fn check(&self, item: &BatchedFederationForecast) -> Vec<String> {
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

/// Runs every rule of a batched federation forecast.
#[derive(Debug, Clone)]
pub struct BatchedFederationForecastValidator {
    pub rules: Vec<Rc<dyn BatchedFederationForecastRule>>,
}

impl BatchedFederationForecastValidator {
    pub fn validate(&self, item: &BatchedFederationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedFederationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFederationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_federation_forecast_validator() -> BatchedFederationForecastValidator {
    BatchedFederationForecastValidator {
        rules: vec![
            Rc::new(BatchedFederationForecastLocaleLengthRule { max_length: 3600 })
                as Rc<dyn BatchedFederationForecastRule>,
            Rc::new(BatchedFederationForecastRegionLengthRule { max_length: 120 }),
            Rc::new(BatchedFederationForecastRequiredLocaleRule {
                message: "batched federation forecast needs a value".to_string(),
            }),
            Rc::new(BatchedFederationForecastThresholdRangeRule { min: 5.4, max: 984.1 }),
        ],
    }
}
