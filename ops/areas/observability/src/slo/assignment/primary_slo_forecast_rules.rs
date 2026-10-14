use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary slo forecast must pass. Check returns its problems, or none.
pub trait PrimarySloForecastRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySloForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloForecastCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySloForecastRule for PrimarySloForecastCodeLengthRule {
    fn check(&self, item: &PrimarySloForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloForecastTitleLengthRule {
    pub max_length: i64,
}

impl PrimarySloForecastRule for PrimarySloForecastTitleLengthRule {
    fn check(&self, item: &PrimarySloForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PrimarySloForecastRule for PrimarySloForecastAllowedCodeRule {
    fn check(&self, item: &PrimarySloForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary slo forecast.
#[derive(Debug, Clone)]
pub struct PrimarySloForecastValidator {
    pub rules: Vec<Rc<dyn PrimarySloForecastRule>>,
}

impl PrimarySloForecastValidator {
    pub fn validate(&self, item: &PrimarySloForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySloForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySloForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_slo_forecast_validator() -> PrimarySloForecastValidator {
    PrimarySloForecastValidator {
        rules: vec![
            Rc::new(PrimarySloForecastCodeLengthRule { max_length: 86400 }) as Rc<dyn PrimarySloForecastRule>,
            Rc::new(PrimarySloForecastTitleLengthRule { max_length: 32 }),
            Rc::new(PrimarySloForecastAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "legacy".to_string(), "default".to_string()],
            }),
        ],
    }
}
