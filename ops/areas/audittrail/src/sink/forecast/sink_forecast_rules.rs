use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink forecast must pass. Check returns its problems, or none.
pub trait SinkForecastRule: std::fmt::Debug {
    fn check(&self, item: &SinkForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkForecastFingerprintLengthRule {
    pub max_length: i64,
}

impl SinkForecastRule for SinkForecastFingerprintLengthRule {
    fn check(&self, item: &SinkForecast) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkForecastLabelLengthRule {
    pub max_length: i64,
}

impl SinkForecastRule for SinkForecastLabelLengthRule {
    fn check(&self, item: &SinkForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkForecastAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SinkForecastRule for SinkForecastAllowedLabelRule {
    fn check(&self, item: &SinkForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkForecastRequiredChannelRule {
    pub message: String,
}

impl SinkForecastRule for SinkForecastRequiredChannelRule {
    fn check(&self, item: &SinkForecast) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink forecast.
#[derive(Debug, Clone)]
pub struct SinkForecastValidator {
    pub rules: Vec<Rc<dyn SinkForecastRule>>,
}

impl SinkForecastValidator {
    pub fn validate(&self, item: &SinkForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_forecast_validator() -> SinkForecastValidator {
    SinkForecastValidator {
        rules: vec![
            Rc::new(SinkForecastFingerprintLengthRule { max_length: 1000 }) as Rc<dyn SinkForecastRule>,
            Rc::new(SinkForecastLabelLengthRule { max_length: 255 }),
            Rc::new(SinkForecastAllowedLabelRule {
                allowed: vec!["default".to_string(), "primary".to_string(), "primary".to_string()],
            }),
            Rc::new(SinkForecastRequiredChannelRule {
                message: "sink forecast needs an entry".to_string(),
            }),
        ],
    }
}
