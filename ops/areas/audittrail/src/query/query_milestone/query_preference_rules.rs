use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query preference must pass. Check returns its problems, or none.
pub trait QueryPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &QueryPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPreferenceLabelLengthRule {
    pub max_length: i64,
}

impl QueryPreferenceRule for QueryPreferenceLabelLengthRule {
    fn check(&self, item: &QueryPreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPreferenceRequiredCodeRule {
    pub message: String,
}

impl QueryPreferenceRule for QueryPreferenceRequiredCodeRule {
    fn check(&self, item: &QueryPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPreferenceIssuerLengthRule {
    pub max_length: i64,
}

impl QueryPreferenceRule for QueryPreferenceIssuerLengthRule {
    fn check(&self, item: &QueryPreference) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryPreferenceAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl QueryPreferenceRule for QueryPreferenceAllowedKeyIdRule {
    fn check(&self, item: &QueryPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is blocked", item.key_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query preference.
#[derive(Debug, Clone)]
pub struct QueryPreferenceValidator {
    pub rules: Vec<Rc<dyn QueryPreferenceRule>>,
}

impl QueryPreferenceValidator {
    pub fn validate(&self, item: &QueryPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QueryPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QueryPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_preference_validator() -> QueryPreferenceValidator {
    QueryPreferenceValidator {
        rules: vec![
            Rc::new(QueryPreferenceLabelLengthRule { max_length: 120 }) as Rc<dyn QueryPreferenceRule>,
            Rc::new(QueryPreferenceRequiredCodeRule {
                message: "query preference needs a value".to_string(),
            }),
            Rc::new(QueryPreferenceIssuerLengthRule { max_length: 255 }),
            Rc::new(QueryPreferenceAllowedKeyIdRule {
                allowed: vec!["pending".to_string(), "pending".to_string()],
            }),
        ],
    }
}
