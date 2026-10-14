use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a forecast preference must pass. Check returns its problems, or none.
pub trait ForecastPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &ForecastPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreferenceRequiredCountryRule {
    pub message: String,
}

impl ForecastPreferenceRule for ForecastPreferenceRequiredCountryRule {
    fn check(&self, item: &ForecastPreference) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreferenceCodePrefixRule {
    pub prefix: String,
}

impl ForecastPreferenceRule for ForecastPreferenceCodePrefixRule {
    fn check(&self, item: &ForecastPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreferenceCountryLengthRule {
    pub max_length: i64,
}

impl ForecastPreferenceRule for ForecastPreferenceCountryLengthRule {
    fn check(&self, item: &ForecastPreference) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreferenceAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ForecastPreferenceRule for ForecastPreferenceAllowedCountryRule {
    fn check(&self, item: &ForecastPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is duplicated", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreferenceRequiredCodeRule {
    pub message: String,
}

impl ForecastPreferenceRule for ForecastPreferenceRequiredCodeRule {
    fn check(&self, item: &ForecastPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl ForecastPreferenceRule for ForecastPreferenceCodeLengthRule {
    fn check(&self, item: &ForecastPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast preference.
#[derive(Debug, Clone)]
pub struct ForecastPreferenceValidator {
    pub rules: Vec<Rc<dyn ForecastPreferenceRule>>,
}

impl ForecastPreferenceValidator {
    pub fn validate(&self, item: &ForecastPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_preference_validator() -> ForecastPreferenceValidator {
    ForecastPreferenceValidator {
        rules: vec![
            Rc::new(ForecastPreferenceRequiredCountryRule {
                message: "forecast preference needs a value".to_string(),
            }) as Rc<dyn ForecastPreferenceRule>,
            Rc::new(ForecastPreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ForecastPreferenceCountryLengthRule { max_length: 120 }),
            Rc::new(ForecastPreferenceAllowedCountryRule {
                allowed: vec!["adaptive".to_string(), "default".to_string(), "weighted".to_string()],
            }),
            Rc::new(ForecastPreferenceRequiredCodeRule {
                message: "forecast preference needs this field".to_string(),
            }),
            Rc::new(ForecastPreferenceCodeLengthRule { max_length: 1000 }),
        ],
    }
}
