use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending slo forecast must pass. Check returns its problems, or none.
pub trait PendingSloForecastRule: std::fmt::Debug {
    fn check(&self, item: &PendingSloForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastCodeLengthRule {
    pub max_length: i64,
}

impl PendingSloForecastRule for PendingSloForecastCodeLengthRule {
    fn check(&self, item: &PendingSloForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastCodePrefixRule {
    pub prefix: String,
}

impl PendingSloForecastRule for PendingSloForecastCodePrefixRule {
    fn check(&self, item: &PendingSloForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastRequiredCategoryRule {
    pub message: String,
}

impl PendingSloForecastRule for PendingSloForecastRequiredCategoryRule {
    fn check(&self, item: &PendingSloForecast) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastTitleLengthRule {
    pub max_length: i64,
}

impl PendingSloForecastRule for PendingSloForecastTitleLengthRule {
    fn check(&self, item: &PendingSloForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastRequiredChecksumRule {
    pub message: String,
}

impl PendingSloForecastRule for PendingSloForecastRequiredChecksumRule {
    fn check(&self, item: &PendingSloForecast) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending slo forecast.
#[derive(Debug, Clone)]
pub struct PendingSloForecastValidator {
    pub rules: Vec<Rc<dyn PendingSloForecastRule>>,
}

impl PendingSloForecastValidator {
    pub fn validate(&self, item: &PendingSloForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingSloForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingSloForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_slo_forecast_validator() -> PendingSloForecastValidator {
    PendingSloForecastValidator {
        rules: vec![
            Rc::new(PendingSloForecastCodeLengthRule { max_length: 120 }) as Rc<dyn PendingSloForecastRule>,
            Rc::new(PendingSloForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PendingSloForecastRequiredCategoryRule {
                message: "pending slo forecast needs a value".to_string(),
            }),
            Rc::new(PendingSloForecastTitleLengthRule { max_length: 255 }),
            Rc::new(PendingSloForecastRequiredChecksumRule {
                message: "pending slo forecast needs an entry".to_string(),
            }),
        ],
    }
}
