use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension exception must pass. Check returns its problems, or none.
pub trait SuspensionExceptionRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionExceptionCodeLengthRule {
    pub max_length: i64,
}

impl SuspensionExceptionRule for SuspensionExceptionCodeLengthRule {
    fn check(&self, item: &SuspensionException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionExceptionRequiredCodeRule {
    pub message: String,
}

impl SuspensionExceptionRule for SuspensionExceptionRequiredCodeRule {
    fn check(&self, item: &SuspensionException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionExceptionNameLengthRule {
    pub max_length: i64,
}

impl SuspensionExceptionRule for SuspensionExceptionNameLengthRule {
    fn check(&self, item: &SuspensionException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension exception.
#[derive(Debug, Clone)]
pub struct SuspensionExceptionValidator {
    pub rules: Vec<Rc<dyn SuspensionExceptionRule>>,
}

impl SuspensionExceptionValidator {
    pub fn validate(&self, item: &SuspensionException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_exception_validator() -> SuspensionExceptionValidator {
    SuspensionExceptionValidator {
        rules: vec![
            Rc::new(SuspensionExceptionCodeLengthRule { max_length: 120 }) as Rc<dyn SuspensionExceptionRule>,
            Rc::new(SuspensionExceptionRequiredCodeRule {
                message: "suspension exception needs a value".to_string(),
            }),
            Rc::new(SuspensionExceptionNameLengthRule { max_length: 120 }),
        ],
    }
}
