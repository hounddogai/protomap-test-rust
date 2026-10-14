use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence forecast must pass. Check returns its problems, or none.
pub trait EvidenceForecastRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecastVisibleGuardRule {
    pub limit: f64,
}

impl EvidenceForecastRule for EvidenceForecastVisibleGuardRule {
    fn check(&self, item: &EvidenceForecast) -> Vec<String> {
        if item.visible && (item.amount > self.limit) {
            return vec![format!("amount {} exceeds {} while visible", item.amount, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecastLabelLengthRule {
    pub max_length: i64,
}

impl EvidenceForecastRule for EvidenceForecastLabelLengthRule {
    fn check(&self, item: &EvidenceForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceForecastCodePrefixRule {
    pub prefix: String,
}

impl EvidenceForecastRule for EvidenceForecastCodePrefixRule {
    fn check(&self, item: &EvidenceForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence forecast.
#[derive(Debug, Clone)]
pub struct EvidenceForecastValidator {
    pub rules: Vec<Rc<dyn EvidenceForecastRule>>,
}

impl EvidenceForecastValidator {
    pub fn validate(&self, item: &EvidenceForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_forecast_validator() -> EvidenceForecastValidator {
    EvidenceForecastValidator {
        rules: vec![
            Rc::new(EvidenceForecastVisibleGuardRule { limit: 9.5 }) as Rc<dyn EvidenceForecastRule>,
            Rc::new(EvidenceForecastLabelLengthRule { max_length: 1000 }),
            Rc::new(EvidenceForecastCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
