use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted identity preference must pass. Check returns its problems, or none.
pub trait WeightedIdentityPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferenceRequiredSourceRule {
    pub message: String,
}

impl WeightedIdentityPreferenceRule for WeightedIdentityPreferenceRequiredSourceRule {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferencePrincipalLengthRule {
    pub max_length: i64,
}

impl WeightedIdentityPreferenceRule for WeightedIdentityPreferencePrincipalLengthRule {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferenceAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl WeightedIdentityPreferenceRule for WeightedIdentityPreferenceAllowedSourceRule {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is too long", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferenceApprovedGuardRule {
    pub limit: i64,
}

impl WeightedIdentityPreferenceRule for WeightedIdentityPreferenceApprovedGuardRule {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String> {
        if item.approved && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while approved", item.burst, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferenceSourceLengthRule {
    pub max_length: i64,
}

impl WeightedIdentityPreferenceRule for WeightedIdentityPreferenceSourceLengthRule {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferenceRequiredNameRule {
    pub message: String,
}

impl WeightedIdentityPreferenceRule for WeightedIdentityPreferenceRequiredNameRule {
    fn check(&self, item: &WeightedIdentityPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted identity preference.
#[derive(Debug, Clone)]
pub struct WeightedIdentityPreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedIdentityPreferenceRule>>,
}

impl WeightedIdentityPreferenceValidator {
    pub fn validate(&self, item: &WeightedIdentityPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedIdentityPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedIdentityPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_identity_preference_validator() -> WeightedIdentityPreferenceValidator {
    WeightedIdentityPreferenceValidator {
        rules: vec![
            Rc::new(WeightedIdentityPreferenceRequiredSourceRule {
                message: "weighted identity preference needs a value".to_string(),
            }) as Rc<dyn WeightedIdentityPreferenceRule>,
            Rc::new(WeightedIdentityPreferencePrincipalLengthRule { max_length: 1000 }),
            Rc::new(WeightedIdentityPreferenceAllowedSourceRule {
                allowed: vec!["rolling".to_string(), "weighted".to_string()],
            }),
            Rc::new(WeightedIdentityPreferenceApprovedGuardRule { limit: 64 }),
            Rc::new(WeightedIdentityPreferenceSourceLengthRule { max_length: 64 }),
            Rc::new(WeightedIdentityPreferenceRequiredNameRule {
                message: "weighted identity preference needs a value".to_string(),
            }),
        ],
    }
}
