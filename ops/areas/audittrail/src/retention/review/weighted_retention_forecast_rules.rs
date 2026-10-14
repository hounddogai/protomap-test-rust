use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted retention forecast must pass. Check returns its problems, or none.
pub trait WeightedRetentionForecastRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRetentionForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionForecastCodePrefixRule {
    pub prefix: String,
}

impl WeightedRetentionForecastRule for WeightedRetentionForecastCodePrefixRule {
    fn check(&self, item: &WeightedRetentionForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedRetentionForecastRule for WeightedRetentionForecastAllowedCodeRule {
    fn check(&self, item: &WeightedRetentionForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionForecastContactEmailLengthRule {
    pub max_length: i64,
}

impl WeightedRetentionForecastRule for WeightedRetentionForecastContactEmailLengthRule {
    fn check(&self, item: &WeightedRetentionForecast) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted retention forecast.
#[derive(Debug, Clone)]
pub struct WeightedRetentionForecastValidator {
    pub rules: Vec<Rc<dyn WeightedRetentionForecastRule>>,
}

impl WeightedRetentionForecastValidator {
    pub fn validate(&self, item: &WeightedRetentionForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRetentionForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRetentionForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_retention_forecast_validator() -> WeightedRetentionForecastValidator {
    WeightedRetentionForecastValidator {
        rules: vec![
            Rc::new(WeightedRetentionForecastCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn WeightedRetentionForecastRule>,
            Rc::new(WeightedRetentionForecastAllowedCodeRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(WeightedRetentionForecastContactEmailLengthRule { max_length: 120 }),
        ],
    }
}
