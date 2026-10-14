use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding cycle must pass. Check returns its problems, or none.
pub trait BindingCycleRule: std::fmt::Debug {
    fn check(&self, item: &BindingCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCycleRequiredCodeRule {
    pub message: String,
}

impl BindingCycleRule for BindingCycleRequiredCodeRule {
    fn check(&self, item: &BindingCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCycleCodePrefixRule {
    pub prefix: String,
}

impl BindingCycleRule for BindingCycleCodePrefixRule {
    fn check(&self, item: &BindingCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingCycleCodeLengthRule {
    pub max_length: i64,
}

impl BindingCycleRule for BindingCycleCodeLengthRule {
    fn check(&self, item: &BindingCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding cycle.
#[derive(Debug, Clone)]
pub struct BindingCycleValidator {
    pub rules: Vec<Rc<dyn BindingCycleRule>>,
}

impl BindingCycleValidator {
    pub fn validate(&self, item: &BindingCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BindingCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_cycle_validator() -> BindingCycleValidator {
    BindingCycleValidator {
        rules: vec![
            Rc::new(BindingCycleRequiredCodeRule {
                message: "binding cycle needs this field".to_string(),
            }) as Rc<dyn BindingCycleRule>,
            Rc::new(BindingCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BindingCycleCodeLengthRule { max_length: 86400 }),
        ],
    }
}
