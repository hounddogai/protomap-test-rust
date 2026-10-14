use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension rollup must pass. Check returns its problems, or none.
pub trait SuspensionRollupRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupRequiredPhoneRule {
    pub message: String,
}

impl SuspensionRollupRule for SuspensionRollupRequiredPhoneRule {
    fn check(&self, item: &SuspensionRollup) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupHostLengthRule {
    pub max_length: i64,
}

impl SuspensionRollupRule for SuspensionRollupHostLengthRule {
    fn check(&self, item: &SuspensionRollup) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupRequiredHostRule {
    pub message: String,
}

impl SuspensionRollupRule for SuspensionRollupRequiredHostRule {
    fn check(&self, item: &SuspensionRollup) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupTitleLengthRule {
    pub max_length: i64,
}

impl SuspensionRollupRule for SuspensionRollupTitleLengthRule {
    fn check(&self, item: &SuspensionRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupCodePrefixRule {
    pub prefix: String,
}

impl SuspensionRollupRule for SuspensionRollupCodePrefixRule {
    fn check(&self, item: &SuspensionRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupPhoneLengthRule {
    pub max_length: i64,
}

impl SuspensionRollupRule for SuspensionRollupPhoneLengthRule {
    fn check(&self, item: &SuspensionRollup) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension rollup.
#[derive(Debug, Clone)]
pub struct SuspensionRollupValidator {
    pub rules: Vec<Rc<dyn SuspensionRollupRule>>,
}

impl SuspensionRollupValidator {
    pub fn validate(&self, item: &SuspensionRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SuspensionRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_rollup_validator() -> SuspensionRollupValidator {
    SuspensionRollupValidator {
        rules: vec![
            Rc::new(SuspensionRollupRequiredPhoneRule {
                message: "suspension rollup needs a value".to_string(),
            }) as Rc<dyn SuspensionRollupRule>,
            Rc::new(SuspensionRollupHostLengthRule { max_length: 86400 }),
            Rc::new(SuspensionRollupRequiredHostRule {
                message: "suspension rollup needs a value".to_string(),
            }),
            Rc::new(SuspensionRollupTitleLengthRule { max_length: 255 }),
            Rc::new(SuspensionRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SuspensionRollupPhoneLengthRule { max_length: 80 }),
        ],
    }
}
