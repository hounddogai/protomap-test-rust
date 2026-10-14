use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy silence exception must pass. Check returns its problems, or none.
pub trait LegacySilenceExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LegacySilenceException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionCodePrefixRule {
    pub prefix: String,
}

impl LegacySilenceExceptionRule for LegacySilenceExceptionCodePrefixRule {
    fn check(&self, item: &LegacySilenceException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionChecksumLengthRule {
    pub max_length: i64,
}

impl LegacySilenceExceptionRule for LegacySilenceExceptionChecksumLengthRule {
    fn check(&self, item: &LegacySilenceException) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionNameLengthRule {
    pub max_length: i64,
}

impl LegacySilenceExceptionRule for LegacySilenceExceptionNameLengthRule {
    fn check(&self, item: &LegacySilenceException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionRequiredCodeRule {
    pub message: String,
}

impl LegacySilenceExceptionRule for LegacySilenceExceptionRequiredCodeRule {
    fn check(&self, item: &LegacySilenceException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionRequiredChecksumRule {
    pub message: String,
}

impl LegacySilenceExceptionRule for LegacySilenceExceptionRequiredChecksumRule {
    fn check(&self, item: &LegacySilenceException) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacySilenceExceptionRule for LegacySilenceExceptionMarginRangeRule {
    fn check(&self, item: &LegacySilenceException) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a legacy silence exception.
#[derive(Debug, Clone)]
pub struct LegacySilenceExceptionValidator {
    pub rules: Vec<Rc<dyn LegacySilenceExceptionRule>>,
}

impl LegacySilenceExceptionValidator {
    pub fn validate(&self, item: &LegacySilenceException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacySilenceException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySilenceException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_silence_exception_validator() -> LegacySilenceExceptionValidator {
    LegacySilenceExceptionValidator {
        rules: vec![
            Rc::new(LegacySilenceExceptionCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn LegacySilenceExceptionRule>,
            Rc::new(LegacySilenceExceptionChecksumLengthRule { max_length: 32 }),
            Rc::new(LegacySilenceExceptionNameLengthRule { max_length: 64 }),
            Rc::new(LegacySilenceExceptionRequiredCodeRule {
                message: "legacy silence exception needs this field".to_string(),
            }),
            Rc::new(LegacySilenceExceptionRequiredChecksumRule {
                message: "legacy silence exception needs this field".to_string(),
            }),
            Rc::new(LegacySilenceExceptionMarginRangeRule { min: 9.1, max: 4526.2 }),
        ],
    }
}
