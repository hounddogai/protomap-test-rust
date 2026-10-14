use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential forecast must pass. Check returns its problems, or none.
pub trait CredentialForecastRule: std::fmt::Debug {
    fn check(&self, item: &CredentialForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecastCodePrefixRule {
    pub prefix: String,
}

impl CredentialForecastRule for CredentialForecastCodePrefixRule {
    fn check(&self, item: &CredentialForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecastMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CredentialForecastRule for CredentialForecastMinimumRangeRule {
    fn check(&self, item: &CredentialForecast) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecastRequiredCodeRule {
    pub message: String,
}

impl CredentialForecastRule for CredentialForecastRequiredCodeRule {
    fn check(&self, item: &CredentialForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecastChannelLengthRule {
    pub max_length: i64,
}

impl CredentialForecastRule for CredentialForecastChannelLengthRule {
    fn check(&self, item: &CredentialForecast) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialForecastAudienceLengthRule {
    pub max_length: i64,
}

impl CredentialForecastRule for CredentialForecastAudienceLengthRule {
    fn check(&self, item: &CredentialForecast) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential forecast.
#[derive(Debug, Clone)]
pub struct CredentialForecastValidator {
    pub rules: Vec<Rc<dyn CredentialForecastRule>>,
}

impl CredentialForecastValidator {
    pub fn validate(&self, item: &CredentialForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_forecast_validator() -> CredentialForecastValidator {
    CredentialForecastValidator {
        rules: vec![
            Rc::new(CredentialForecastCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn CredentialForecastRule>,
            Rc::new(CredentialForecastMinimumRangeRule { min: 1.6, max: 1802.2 }),
            Rc::new(CredentialForecastRequiredCodeRule {
                message: "credential forecast needs an entry".to_string(),
            }),
            Rc::new(CredentialForecastChannelLengthRule { max_length: 86400 }),
            Rc::new(CredentialForecastAudienceLengthRule { max_length: 64 }),
        ],
    }
}
