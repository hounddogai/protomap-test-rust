use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive release tier must pass. Check returns its problems, or none.
pub trait AdaptiveReleaseTierRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveReleaseTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTierNamespaceLengthRule {
    pub max_length: i64,
}

impl AdaptiveReleaseTierRule for AdaptiveReleaseTierNamespaceLengthRule {
    fn check(&self, item: &AdaptiveReleaseTier) -> Vec<String> {
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
pub struct AdaptiveReleaseTierRequiredCodeRule {
    pub message: String,
}

impl AdaptiveReleaseTierRule for AdaptiveReleaseTierRequiredCodeRule {
    fn check(&self, item: &AdaptiveReleaseTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveReleaseTierRule for AdaptiveReleaseTierAllowedCodeRule {
    fn check(&self, item: &AdaptiveReleaseTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTierLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveReleaseTierRule for AdaptiveReleaseTierLabelLengthRule {
    fn check(&self, item: &AdaptiveReleaseTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTierCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveReleaseTierRule for AdaptiveReleaseTierCodeLengthRule {
    fn check(&self, item: &AdaptiveReleaseTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive release tier.
#[derive(Debug, Clone)]
pub struct AdaptiveReleaseTierValidator {
    pub rules: Vec<Rc<dyn AdaptiveReleaseTierRule>>,
}

impl AdaptiveReleaseTierValidator {
    pub fn validate(&self, item: &AdaptiveReleaseTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveReleaseTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveReleaseTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_release_tier_validator() -> AdaptiveReleaseTierValidator {
    AdaptiveReleaseTierValidator {
        rules: vec![
            Rc::new(AdaptiveReleaseTierNamespaceLengthRule { max_length: 255 }) as Rc<dyn AdaptiveReleaseTierRule>,
            Rc::new(AdaptiveReleaseTierRequiredCodeRule {
                message: "adaptive release tier needs an entry".to_string(),
            }),
            Rc::new(AdaptiveReleaseTierAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "adaptive".to_string(), "primary".to_string()],
            }),
            Rc::new(AdaptiveReleaseTierLabelLengthRule { max_length: 120 }),
            Rc::new(AdaptiveReleaseTierCodeLengthRule { max_length: 80 }),
        ],
    }
}
