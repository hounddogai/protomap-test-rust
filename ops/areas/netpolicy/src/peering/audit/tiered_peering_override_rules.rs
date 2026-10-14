use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered peering override must pass. Check returns its problems, or none.
pub trait TieredPeeringOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideRequiredCodeRule {
    pub message: String,
}

impl TieredPeeringOverrideRule for TieredPeeringOverrideRequiredCodeRule {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideRequiredScopeRule {
    pub message: String,
}

impl TieredPeeringOverrideRule for TieredPeeringOverrideRequiredScopeRule {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideScopeLengthRule {
    pub max_length: i64,
}

impl TieredPeeringOverrideRule for TieredPeeringOverrideScopeLengthRule {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideCodePrefixRule {
    pub prefix: String,
}

impl TieredPeeringOverrideRule for TieredPeeringOverrideCodePrefixRule {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideCodeLengthRule {
    pub max_length: i64,
}

impl TieredPeeringOverrideRule for TieredPeeringOverrideCodeLengthRule {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringOverrideRequiredLabelRule {
    pub message: String,
}

impl TieredPeeringOverrideRule for TieredPeeringOverrideRequiredLabelRule {
    fn check(&self, item: &TieredPeeringOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered peering override.
#[derive(Debug, Clone)]
pub struct TieredPeeringOverrideValidator {
    pub rules: Vec<Rc<dyn TieredPeeringOverrideRule>>,
}

impl TieredPeeringOverrideValidator {
    pub fn validate(&self, item: &TieredPeeringOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredPeeringOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredPeeringOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_peering_override_validator() -> TieredPeeringOverrideValidator {
    TieredPeeringOverrideValidator {
        rules: vec![
            Rc::new(TieredPeeringOverrideRequiredCodeRule {
                message: "tiered peering override needs an entry".to_string(),
            }) as Rc<dyn TieredPeeringOverrideRule>,
            Rc::new(TieredPeeringOverrideRequiredScopeRule {
                message: "tiered peering override needs a value".to_string(),
            }),
            Rc::new(TieredPeeringOverrideScopeLengthRule { max_length: 3600 }),
            Rc::new(TieredPeeringOverrideCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TieredPeeringOverrideCodeLengthRule { max_length: 80 }),
            Rc::new(TieredPeeringOverrideRequiredLabelRule {
                message: "tiered peering override needs a value".to_string(),
            }),
        ],
    }
}
