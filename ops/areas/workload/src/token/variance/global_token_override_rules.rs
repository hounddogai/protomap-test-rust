use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global token override must pass. Check returns its problems, or none.
pub trait GlobalTokenOverrideRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTokenOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideCodePrefixRule {
    pub prefix: String,
}

impl GlobalTokenOverrideRule for GlobalTokenOverrideCodePrefixRule {
    fn check(&self, item: &GlobalTokenOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideRequiredLabelRule {
    pub message: String,
}

impl GlobalTokenOverrideRule for GlobalTokenOverrideRequiredLabelRule {
    fn check(&self, item: &GlobalTokenOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalTokenOverrideRule for GlobalTokenOverrideAllowedCodeRule {
    fn check(&self, item: &GlobalTokenOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTokenOverrideRule for GlobalTokenOverrideQuantityRangeRule {
    fn check(&self, item: &GlobalTokenOverride) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global token override.
#[derive(Debug, Clone)]
pub struct GlobalTokenOverrideValidator {
    pub rules: Vec<Rc<dyn GlobalTokenOverrideRule>>,
}

impl GlobalTokenOverrideValidator {
    pub fn validate(&self, item: &GlobalTokenOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalTokenOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTokenOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_token_override_validator() -> GlobalTokenOverrideValidator {
    GlobalTokenOverrideValidator {
        rules: vec![
            Rc::new(GlobalTokenOverrideCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn GlobalTokenOverrideRule>,
            Rc::new(GlobalTokenOverrideRequiredLabelRule {
                message: "global token override needs a value".to_string(),
            }),
            Rc::new(GlobalTokenOverrideAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "batched".to_string()],
            }),
            Rc::new(GlobalTokenOverrideQuantityRangeRule { min: 255, max: 255 }),
        ],
    }
}
