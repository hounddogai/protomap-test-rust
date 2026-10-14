use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback runbook forecast must pass. Check returns its problems, or none.
pub trait FallbackRunbookForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRunbookForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookForecastTitleLengthRule {
    pub max_length: i64,
}

impl FallbackRunbookForecastRule for FallbackRunbookForecastTitleLengthRule {
    fn check(&self, item: &FallbackRunbookForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookForecastRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRunbookForecastRule for FallbackRunbookForecastRevisionRangeRule {
    fn check(&self, item: &FallbackRunbookForecast) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookForecastRequiredTitleRule {
    pub message: String,
}

impl FallbackRunbookForecastRule for FallbackRunbookForecastRequiredTitleRule {
    fn check(&self, item: &FallbackRunbookForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback runbook forecast.
#[derive(Debug, Clone)]
pub struct FallbackRunbookForecastValidator {
    pub rules: Vec<Rc<dyn FallbackRunbookForecastRule>>,
}

impl FallbackRunbookForecastValidator {
    pub fn validate(&self, item: &FallbackRunbookForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackRunbookForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRunbookForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_runbook_forecast_validator() -> FallbackRunbookForecastValidator {
    FallbackRunbookForecastValidator {
        rules: vec![
            Rc::new(FallbackRunbookForecastTitleLengthRule { max_length: 3600 }) as Rc<dyn FallbackRunbookForecastRule>,
            Rc::new(FallbackRunbookForecastRevisionRangeRule { min: 32, max: 3600 }),
            Rc::new(FallbackRunbookForecastRequiredTitleRule {
                message: "fallback runbook forecast needs an entry".to_string(),
            }),
        ],
    }
}
