use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback rule cycle must pass. Check returns its problems, or none.
pub trait FallbackRuleCycleRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRuleCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycleTitleLengthRule {
    pub max_length: i64,
}

impl FallbackRuleCycleRule for FallbackRuleCycleTitleLengthRule {
    fn check(&self, item: &FallbackRuleCycle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycleCodePrefixRule {
    pub prefix: String,
}

impl FallbackRuleCycleRule for FallbackRuleCycleCodePrefixRule {
    fn check(&self, item: &FallbackRuleCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycleRequiredLabelRule {
    pub message: String,
}

impl FallbackRuleCycleRule for FallbackRuleCycleRequiredLabelRule {
    fn check(&self, item: &FallbackRuleCycle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycleRequiredNamespaceRule {
    pub message: String,
}

impl FallbackRuleCycleRule for FallbackRuleCycleRequiredNamespaceRule {
    fn check(&self, item: &FallbackRuleCycle) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleCycleCodeLengthRule {
    pub max_length: i64,
}

impl FallbackRuleCycleRule for FallbackRuleCycleCodeLengthRule {
    fn check(&self, item: &FallbackRuleCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback rule cycle.
#[derive(Debug, Clone)]
pub struct FallbackRuleCycleValidator {
    pub rules: Vec<Rc<dyn FallbackRuleCycleRule>>,
}

impl FallbackRuleCycleValidator {
    pub fn validate(&self, item: &FallbackRuleCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRuleCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRuleCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_rule_cycle_validator() -> FallbackRuleCycleValidator {
    FallbackRuleCycleValidator {
        rules: vec![
            Rc::new(FallbackRuleCycleTitleLengthRule { max_length: 120 }) as Rc<dyn FallbackRuleCycleRule>,
            Rc::new(FallbackRuleCycleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(FallbackRuleCycleRequiredLabelRule {
                message: "fallback rule cycle needs a value".to_string(),
            }),
            Rc::new(FallbackRuleCycleRequiredNamespaceRule {
                message: "fallback rule cycle needs an entry".to_string(),
            }),
            Rc::new(FallbackRuleCycleCodeLengthRule { max_length: 120 }),
        ],
    }
}
