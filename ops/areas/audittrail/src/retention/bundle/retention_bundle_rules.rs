use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention bundle must pass. Check returns its problems, or none.
pub trait RetentionBundleRule: std::fmt::Debug {
    fn check(&self, item: &RetentionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleAllowedHostRule {
    pub allowed: Vec<String>,
}

impl RetentionBundleRule for RetentionBundleAllowedHostRule {
    fn check(&self, item: &RetentionBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is not allowed", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleCodePrefixRule {
    pub prefix: String,
}

impl RetentionBundleRule for RetentionBundleCodePrefixRule {
    fn check(&self, item: &RetentionBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionBundleRule for RetentionBundleAmountRangeRule {
    fn check(&self, item: &RetentionBundle) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleCodeLengthRule {
    pub max_length: i64,
}

impl RetentionBundleRule for RetentionBundleCodeLengthRule {
    fn check(&self, item: &RetentionBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleRequiredOwnerIdRule {
    pub message: String,
}

impl RetentionBundleRule for RetentionBundleRequiredOwnerIdRule {
    fn check(&self, item: &RetentionBundle) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBundleOwnerIdLengthRule {
    pub max_length: i64,
}

impl RetentionBundleRule for RetentionBundleOwnerIdLengthRule {
    fn check(&self, item: &RetentionBundle) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention bundle.
#[derive(Debug, Clone)]
pub struct RetentionBundleValidator {
    pub rules: Vec<Rc<dyn RetentionBundleRule>>,
}

impl RetentionBundleValidator {
    pub fn validate(&self, item: &RetentionBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_bundle_validator() -> RetentionBundleValidator {
    RetentionBundleValidator {
        rules: vec![
            Rc::new(RetentionBundleAllowedHostRule {
                allowed: vec!["primary".to_string(), "regional".to_string()],
            }) as Rc<dyn RetentionBundleRule>,
            Rc::new(RetentionBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RetentionBundleAmountRangeRule { min: 3.3, max: 3091.8 }),
            Rc::new(RetentionBundleCodeLengthRule { max_length: 64 }),
            Rc::new(RetentionBundleRequiredOwnerIdRule {
                message: "retention bundle needs an entry".to_string(),
            }),
            Rc::new(RetentionBundleOwnerIdLengthRule { max_length: 1000 }),
        ],
    }
}
