use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert threshold must pass. Check returns its problems, or none.
pub trait AlertThresholdRule: std::fmt::Debug {
    fn check(&self, item: &AlertThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertThresholdCodePrefixRule {
    pub prefix: String,
}

impl AlertThresholdRule for AlertThresholdCodePrefixRule {
    fn check(&self, item: &AlertThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertThresholdNamespaceLengthRule {
    pub max_length: i64,
}

impl AlertThresholdRule for AlertThresholdNamespaceLengthRule {
    fn check(&self, item: &AlertThreshold) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertThresholdAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl AlertThresholdRule for AlertThresholdAllowedNamespaceRule {
    fn check(&self, item: &AlertThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is inconsistent", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertThresholdApprovedGuardRule {
    pub limit: f64,
}

impl AlertThresholdRule for AlertThresholdApprovedGuardRule {
    fn check(&self, item: &AlertThreshold) -> Vec<String> {
        if item.approved && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while approved",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertThresholdTitleLengthRule {
    pub max_length: i64,
}

impl AlertThresholdRule for AlertThresholdTitleLengthRule {
    fn check(&self, item: &AlertThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert threshold.
#[derive(Debug, Clone)]
pub struct AlertThresholdValidator {
    pub rules: Vec<Rc<dyn AlertThresholdRule>>,
}

impl AlertThresholdValidator {
    pub fn validate(&self, item: &AlertThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_threshold_validator() -> AlertThresholdValidator {
    AlertThresholdValidator {
        rules: vec![
            Rc::new(AlertThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AlertThresholdRule>,
            Rc::new(AlertThresholdNamespaceLengthRule { max_length: 120 }),
            Rc::new(AlertThresholdAllowedNamespaceRule {
                allowed: vec!["regional".to_string()],
            }),
            Rc::new(AlertThresholdApprovedGuardRule { limit: 4.0 }),
            Rc::new(AlertThresholdTitleLengthRule { max_length: 80 }),
        ],
    }
}
