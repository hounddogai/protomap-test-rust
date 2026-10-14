use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default action item target must pass. Check returns its problems, or none.
pub trait DefaultActionItemTargetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultActionItemTargetRule for DefaultActionItemTargetUpdatedAtRangeRule {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetCodePrefixRule {
    pub prefix: String,
}

impl DefaultActionItemTargetRule for DefaultActionItemTargetCodePrefixRule {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetCodeLengthRule {
    pub max_length: i64,
}

impl DefaultActionItemTargetRule for DefaultActionItemTargetCodeLengthRule {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetDescriptionLengthRule {
    pub max_length: i64,
}

impl DefaultActionItemTargetRule for DefaultActionItemTargetDescriptionLengthRule {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl DefaultActionItemTargetRule for DefaultActionItemTargetAllowedNameRule {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetRequiredNameRule {
    pub message: String,
}

impl DefaultActionItemTargetRule for DefaultActionItemTargetRequiredNameRule {
    fn check(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default action item target.
#[derive(Debug, Clone)]
pub struct DefaultActionItemTargetValidator {
    pub rules: Vec<Rc<dyn DefaultActionItemTargetRule>>,
}

impl DefaultActionItemTargetValidator {
    pub fn validate(&self, item: &DefaultActionItemTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultActionItemTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultActionItemTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_action_item_target_validator() -> DefaultActionItemTargetValidator {
    DefaultActionItemTargetValidator {
        rules: vec![
            Rc::new(DefaultActionItemTargetUpdatedAtRangeRule { min: 80, max: 120 })
                as Rc<dyn DefaultActionItemTargetRule>,
            Rc::new(DefaultActionItemTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultActionItemTargetCodeLengthRule { max_length: 80 }),
            Rc::new(DefaultActionItemTargetDescriptionLengthRule { max_length: 32 }),
            Rc::new(DefaultActionItemTargetAllowedNameRule {
                allowed: vec!["lenient".to_string(), "primary".to_string()],
            }),
            Rc::new(DefaultActionItemTargetRequiredNameRule {
                message: "default action item target needs an entry".to_string(),
            }),
        ],
    }
}
