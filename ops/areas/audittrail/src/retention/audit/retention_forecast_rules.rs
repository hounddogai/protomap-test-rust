use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention forecast must pass. Check returns its problems, or none.
pub trait RetentionForecastRule: std::fmt::Debug {
    fn check(&self, item: &RetentionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecastAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RetentionForecastRule for RetentionForecastAllowedNameRule {
    fn check(&self, item: &RetentionForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecastRequiredLabelRule {
    pub message: String,
}

impl RetentionForecastRule for RetentionForecastRequiredLabelRule {
    fn check(&self, item: &RetentionForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecastRequiredCodeRule {
    pub message: String,
}

impl RetentionForecastRule for RetentionForecastRequiredCodeRule {
    fn check(&self, item: &RetentionForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionForecastCodePrefixRule {
    pub prefix: String,
}

impl RetentionForecastRule for RetentionForecastCodePrefixRule {
    fn check(&self, item: &RetentionForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention forecast.
#[derive(Debug, Clone)]
pub struct RetentionForecastValidator {
    pub rules: Vec<Rc<dyn RetentionForecastRule>>,
}

impl RetentionForecastValidator {
    pub fn validate(&self, item: &RetentionForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_forecast_validator() -> RetentionForecastValidator {
    RetentionForecastValidator {
        rules: vec![
            Rc::new(RetentionForecastAllowedNameRule {
                allowed: vec!["default".to_string(), "weighted".to_string()],
            }) as Rc<dyn RetentionForecastRule>,
            Rc::new(RetentionForecastRequiredLabelRule {
                message: "retention forecast needs a value".to_string(),
            }),
            Rc::new(RetentionForecastRequiredCodeRule {
                message: "retention forecast needs a value".to_string(),
            }),
            Rc::new(RetentionForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
