use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional egress exception must pass. Check returns its problems, or none.
pub trait RegionalEgressExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RegionalEgressException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionCodePrefixRule {
    pub prefix: String,
}

impl RegionalEgressExceptionRule for RegionalEgressExceptionCodePrefixRule {
    fn check(&self, item: &RegionalEgressException) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionRequiredPhoneRule {
    pub message: String,
}

impl RegionalEgressExceptionRule for RegionalEgressExceptionRequiredPhoneRule {
    fn check(&self, item: &RegionalEgressException) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionRequiredHostRule {
    pub message: String,
}

impl RegionalEgressExceptionRule for RegionalEgressExceptionRequiredHostRule {
    fn check(&self, item: &RegionalEgressException) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionRequiredNameRule {
    pub message: String,
}

impl RegionalEgressExceptionRule for RegionalEgressExceptionRequiredNameRule {
    fn check(&self, item: &RegionalEgressException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEgressExceptionHostLengthRule {
    pub max_length: i64,
}

impl RegionalEgressExceptionRule for RegionalEgressExceptionHostLengthRule {
    fn check(&self, item: &RegionalEgressException) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional egress exception.
#[derive(Debug, Clone)]
pub struct RegionalEgressExceptionValidator {
    pub rules: Vec<Rc<dyn RegionalEgressExceptionRule>>,
}

impl RegionalEgressExceptionValidator {
    pub fn validate(&self, item: &RegionalEgressException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalEgressException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalEgressException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_egress_exception_validator() -> RegionalEgressExceptionValidator {
    RegionalEgressExceptionValidator {
        rules: vec![
            Rc::new(RegionalEgressExceptionCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RegionalEgressExceptionRule>,
            Rc::new(RegionalEgressExceptionRequiredPhoneRule {
                message: "regional egress exception needs this field".to_string(),
            }),
            Rc::new(RegionalEgressExceptionRequiredHostRule {
                message: "regional egress exception needs a value".to_string(),
            }),
            Rc::new(RegionalEgressExceptionRequiredNameRule {
                message: "regional egress exception needs a value".to_string(),
            }),
            Rc::new(RegionalEgressExceptionHostLengthRule { max_length: 1000 }),
        ],
    }
}
