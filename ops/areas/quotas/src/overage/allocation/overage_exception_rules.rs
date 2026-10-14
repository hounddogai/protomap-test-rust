use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage exception must pass. Check returns its problems, or none.
pub trait OverageExceptionRule: std::fmt::Debug {
    fn check(&self, item: &OverageException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageExceptionCodeLengthRule {
    pub max_length: i64,
}

impl OverageExceptionRule for OverageExceptionCodeLengthRule {
    fn check(&self, item: &OverageException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageExceptionKeyIdLengthRule {
    pub max_length: i64,
}

impl OverageExceptionRule for OverageExceptionKeyIdLengthRule {
    fn check(&self, item: &OverageException) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageExceptionRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageExceptionRule for OverageExceptionRotationDaysRangeRule {
    fn check(&self, item: &OverageException) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageExceptionMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageExceptionRule for OverageExceptionMaximumRangeRule {
    fn check(&self, item: &OverageException) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage exception.
#[derive(Debug, Clone)]
pub struct OverageExceptionValidator {
    pub rules: Vec<Rc<dyn OverageExceptionRule>>,
}

impl OverageExceptionValidator {
    pub fn validate(&self, item: &OverageException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_exception_validator() -> OverageExceptionValidator {
    OverageExceptionValidator {
        rules: vec![
            Rc::new(OverageExceptionCodeLengthRule { max_length: 80 }) as Rc<dyn OverageExceptionRule>,
            Rc::new(OverageExceptionKeyIdLengthRule { max_length: 120 }),
            Rc::new(OverageExceptionRotationDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(OverageExceptionMaximumRangeRule { min: 8.2, max: 2109.1 }),
        ],
    }
}
