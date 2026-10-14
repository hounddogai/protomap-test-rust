use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual page forecast must pass. Check returns its problems, or none.
pub trait ManualPageForecastRule: std::fmt::Debug {
    fn check(&self, item: &ManualPageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageForecastRequiredTitleRule {
    pub message: String,
}

impl ManualPageForecastRule for ManualPageForecastRequiredTitleRule {
    fn check(&self, item: &ManualPageForecast) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageForecastMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualPageForecastRule for ManualPageForecastMarginRangeRule {
    fn check(&self, item: &ManualPageForecast) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageForecastTitleLengthRule {
    pub max_length: i64,
}

impl ManualPageForecastRule for ManualPageForecastTitleLengthRule {
    fn check(&self, item: &ManualPageForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageForecastAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualPageForecastRule for ManualPageForecastAmountRangeRule {
    fn check(&self, item: &ManualPageForecast) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageForecastAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl ManualPageForecastRule for ManualPageForecastAllowedRegionRule {
    fn check(&self, item: &ManualPageForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is unknown", item.region)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual page forecast.
#[derive(Debug, Clone)]
pub struct ManualPageForecastValidator {
    pub rules: Vec<Rc<dyn ManualPageForecastRule>>,
}

impl ManualPageForecastValidator {
    pub fn validate(&self, item: &ManualPageForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualPageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualPageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_page_forecast_validator() -> ManualPageForecastValidator {
    ManualPageForecastValidator {
        rules: vec![
            Rc::new(ManualPageForecastRequiredTitleRule {
                message: "manual page forecast needs an entry".to_string(),
            }) as Rc<dyn ManualPageForecastRule>,
            Rc::new(ManualPageForecastMarginRangeRule { min: 8.4, max: 1819.8 }),
            Rc::new(ManualPageForecastTitleLengthRule { max_length: 86400 }),
            Rc::new(ManualPageForecastAmountRangeRule { min: 6.1, max: 4299.0 }),
            Rc::new(ManualPageForecastAllowedRegionRule {
                allowed: vec!["batched".to_string(), "default".to_string(), "manual".to_string()],
            }),
        ],
    }
}
