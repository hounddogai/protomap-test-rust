use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback freeze window must pass. Check returns its problems, or none.
pub trait FallbackFreezeWindowRule: std::fmt::Debug {
    fn check(&self, item: &FallbackFreezeWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeWindowRequiredKeyIdRule {
    pub message: String,
}

impl FallbackFreezeWindowRule for FallbackFreezeWindowRequiredKeyIdRule {
    fn check(&self, item: &FallbackFreezeWindow) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeWindowKeyIdLengthRule {
    pub max_length: i64,
}

impl FallbackFreezeWindowRule for FallbackFreezeWindowKeyIdLengthRule {
    fn check(&self, item: &FallbackFreezeWindow) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeWindowCodePrefixRule {
    pub prefix: String,
}

impl FallbackFreezeWindowRule for FallbackFreezeWindowCodePrefixRule {
    fn check(&self, item: &FallbackFreezeWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback freeze window.
#[derive(Debug, Clone)]
pub struct FallbackFreezeWindowValidator {
    pub rules: Vec<Rc<dyn FallbackFreezeWindowRule>>,
}

impl FallbackFreezeWindowValidator {
    pub fn validate(&self, item: &FallbackFreezeWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackFreezeWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackFreezeWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_freeze_window_validator() -> FallbackFreezeWindowValidator {
    FallbackFreezeWindowValidator {
        rules: vec![
            Rc::new(FallbackFreezeWindowRequiredKeyIdRule {
                message: "fallback freeze window needs a value".to_string(),
            }) as Rc<dyn FallbackFreezeWindowRule>,
            Rc::new(FallbackFreezeWindowKeyIdLengthRule { max_length: 255 }),
            Rc::new(FallbackFreezeWindowCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
