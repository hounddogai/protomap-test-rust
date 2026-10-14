use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient action item override must pass. Check returns its problems, or none.
pub trait LenientActionItemOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LenientActionItemOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverrideRequiredOwnerIdRule {
    pub message: String,
}

impl LenientActionItemOverrideRule for LenientActionItemOverrideRequiredOwnerIdRule {
    fn check(&self, item: &LenientActionItemOverride) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverrideCodeLengthRule {
    pub max_length: i64,
}

impl LenientActionItemOverrideRule for LenientActionItemOverrideCodeLengthRule {
    fn check(&self, item: &LenientActionItemOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverrideRequiredCodeRule {
    pub message: String,
}

impl LenientActionItemOverrideRule for LenientActionItemOverrideRequiredCodeRule {
    fn check(&self, item: &LenientActionItemOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverrideRequiredCipherRule {
    pub message: String,
}

impl LenientActionItemOverrideRule for LenientActionItemOverrideRequiredCipherRule {
    fn check(&self, item: &LenientActionItemOverride) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverrideTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl LenientActionItemOverrideRule for LenientActionItemOverrideTagsLimitRule {
    fn check(&self, item: &LenientActionItemOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a lenient action item override.
#[derive(Debug, Clone)]
pub struct LenientActionItemOverrideValidator {
    pub rules: Vec<Rc<dyn LenientActionItemOverrideRule>>,
}

impl LenientActionItemOverrideValidator {
    pub fn validate(&self, item: &LenientActionItemOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientActionItemOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientActionItemOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_action_item_override_validator() -> LenientActionItemOverrideValidator {
    LenientActionItemOverrideValidator {
        rules: vec![
            Rc::new(LenientActionItemOverrideRequiredOwnerIdRule {
                message: "lenient action item override needs an entry".to_string(),
            }) as Rc<dyn LenientActionItemOverrideRule>,
            Rc::new(LenientActionItemOverrideCodeLengthRule { max_length: 120 }),
            Rc::new(LenientActionItemOverrideRequiredCodeRule {
                message: "lenient action item override needs a value".to_string(),
            }),
            Rc::new(LenientActionItemOverrideRequiredCipherRule {
                message: "lenient action item override needs this field".to_string(),
            }),
            Rc::new(LenientActionItemOverrideTagsLimitRule {
                max_count: 16,
                banned: vec!["adaptive".to_string(), "rolling".to_string(), "tiered".to_string()],
            }),
        ],
    }
}
