use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a timeline forecast must pass. Check returns its problems, or none.
pub trait TimelineForecastRule: std::fmt::Debug {
    fn check(&self, item: &TimelineForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineForecastTimezoneLengthRule {
    pub max_length: i64,
}

impl TimelineForecastRule for TimelineForecastTimezoneLengthRule {
    fn check(&self, item: &TimelineForecast) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineForecastRequiredTimezoneRule {
    pub message: String,
}

impl TimelineForecastRule for TimelineForecastRequiredTimezoneRule {
    fn check(&self, item: &TimelineForecast) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineForecastCodePrefixRule {
    pub prefix: String,
}

impl TimelineForecastRule for TimelineForecastCodePrefixRule {
    fn check(&self, item: &TimelineForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineForecastRequiredTitleRule {
    pub message: String,
}

impl TimelineForecastRule for TimelineForecastRequiredTitleRule {
    fn check(&self, item: &TimelineForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline forecast.
#[derive(Debug, Clone)]
pub struct TimelineForecastValidator {
    pub rules: Vec<Rc<dyn TimelineForecastRule>>,
}

impl TimelineForecastValidator {
    pub fn validate(&self, item: &TimelineForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_forecast_validator() -> TimelineForecastValidator {
    TimelineForecastValidator {
        rules: vec![
            Rc::new(TimelineForecastTimezoneLengthRule { max_length: 32 }) as Rc<dyn TimelineForecastRule>,
            Rc::new(TimelineForecastRequiredTimezoneRule {
                message: "timeline forecast needs an entry".to_string(),
            }),
            Rc::new(TimelineForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TimelineForecastRequiredTitleRule {
                message: "timeline forecast needs an entry".to_string(),
            }),
        ],
    }
}
