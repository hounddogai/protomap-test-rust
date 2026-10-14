use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback page adjustment must pass. Check returns its problems, or none.
pub trait FallbackPageAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPageAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl FallbackPageAdjustmentRule for FallbackPageAdjustmentCodeLengthRule {
    fn check(&self, item: &FallbackPageAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl FallbackPageAdjustmentRule for FallbackPageAdjustmentCodePrefixRule {
    fn check(&self, item: &FallbackPageAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl FallbackPageAdjustmentRule for FallbackPageAdjustmentNameLengthRule {
    fn check(&self, item: &FallbackPageAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback page adjustment.
#[derive(Debug, Clone)]
pub struct FallbackPageAdjustmentValidator {
    pub rules: Vec<Rc<dyn FallbackPageAdjustmentRule>>,
}

impl FallbackPageAdjustmentValidator {
    pub fn validate(&self, item: &FallbackPageAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackPageAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPageAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_page_adjustment_validator() -> FallbackPageAdjustmentValidator {
    FallbackPageAdjustmentValidator {
        rules: vec![
            Rc::new(FallbackPageAdjustmentCodeLengthRule { max_length: 64 }) as Rc<dyn FallbackPageAdjustmentRule>,
            Rc::new(FallbackPageAdjustmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FallbackPageAdjustmentNameLengthRule { max_length: 1000 }),
        ],
    }
}
