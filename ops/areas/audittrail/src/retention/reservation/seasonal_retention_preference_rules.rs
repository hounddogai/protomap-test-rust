use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal retention preference must pass. Check returns its problems, or none.
pub trait SeasonalRetentionPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceRequiredCodeRule {
    pub message: String,
}

impl SeasonalRetentionPreferenceRule for SeasonalRetentionPreferenceRequiredCodeRule {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalRetentionPreferenceRule for SeasonalRetentionPreferenceGraceDaysRangeRule {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceScopeLengthRule {
    pub max_length: i64,
}

impl SeasonalRetentionPreferenceRule for SeasonalRetentionPreferenceScopeLengthRule {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl SeasonalRetentionPreferenceRule for SeasonalRetentionPreferenceAllowedScopeRule {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is inconsistent", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceNameLengthRule {
    pub max_length: i64,
}

impl SeasonalRetentionPreferenceRule for SeasonalRetentionPreferenceNameLengthRule {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceCodePrefixRule {
    pub prefix: String,
}

impl SeasonalRetentionPreferenceRule for SeasonalRetentionPreferenceCodePrefixRule {
    fn check(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal retention preference.
#[derive(Debug, Clone)]
pub struct SeasonalRetentionPreferenceValidator {
    pub rules: Vec<Rc<dyn SeasonalRetentionPreferenceRule>>,
}

impl SeasonalRetentionPreferenceValidator {
    pub fn validate(&self, item: &SeasonalRetentionPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalRetentionPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalRetentionPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_retention_preference_validator() -> SeasonalRetentionPreferenceValidator {
    SeasonalRetentionPreferenceValidator {
        rules: vec![
            Rc::new(SeasonalRetentionPreferenceRequiredCodeRule {
                message: "seasonal retention preference needs a value".to_string(),
            }) as Rc<dyn SeasonalRetentionPreferenceRule>,
            Rc::new(SeasonalRetentionPreferenceGraceDaysRangeRule { min: 32, max: 86400 }),
            Rc::new(SeasonalRetentionPreferenceScopeLengthRule { max_length: 64 }),
            Rc::new(SeasonalRetentionPreferenceAllowedScopeRule { allowed: Vec::new() }),
            Rc::new(SeasonalRetentionPreferenceNameLengthRule { max_length: 64 }),
            Rc::new(SeasonalRetentionPreferenceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
