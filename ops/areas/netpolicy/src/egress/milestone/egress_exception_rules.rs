use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress exception must pass. Check returns its problems, or none.
pub trait EgressExceptionRule: std::fmt::Debug {
    fn check(&self, item: &EgressException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressExceptionTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl EgressExceptionRule for EgressExceptionTagsLimitRule {
    fn check(&self, item: &EgressException) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct EgressExceptionRequiredCodeRule {
    pub message: String,
}

impl EgressExceptionRule for EgressExceptionRequiredCodeRule {
    fn check(&self, item: &EgressException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressExceptionTitleLengthRule {
    pub max_length: i64,
}

impl EgressExceptionRule for EgressExceptionTitleLengthRule {
    fn check(&self, item: &EgressException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressExceptionCodeLengthRule {
    pub max_length: i64,
}

impl EgressExceptionRule for EgressExceptionCodeLengthRule {
    fn check(&self, item: &EgressException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressExceptionAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl EgressExceptionRule for EgressExceptionAllowedUnitRule {
    fn check(&self, item: &EgressException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is out of range", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress exception.
#[derive(Debug, Clone)]
pub struct EgressExceptionValidator {
    pub rules: Vec<Rc<dyn EgressExceptionRule>>,
}

impl EgressExceptionValidator {
    pub fn validate(&self, item: &EgressException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_exception_validator() -> EgressExceptionValidator {
    EgressExceptionValidator {
        rules: vec![
            Rc::new(EgressExceptionTagsLimitRule {
                max_count: 80,
                banned: Vec::new(),
            }) as Rc<dyn EgressExceptionRule>,
            Rc::new(EgressExceptionRequiredCodeRule {
                message: "egress exception needs this field".to_string(),
            }),
            Rc::new(EgressExceptionTitleLengthRule { max_length: 16 }),
            Rc::new(EgressExceptionCodeLengthRule { max_length: 120 }),
            Rc::new(EgressExceptionAllowedUnitRule {
                allowed: vec!["scheduled".to_string(), "strict".to_string()],
            }),
        ],
    }
}
