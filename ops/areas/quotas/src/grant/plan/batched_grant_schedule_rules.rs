use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched grant schedule must pass. Check returns its problems, or none.
pub trait BatchedGrantScheduleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedGrantSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantScheduleCodePrefixRule {
    pub prefix: String,
}

impl BatchedGrantScheduleRule for BatchedGrantScheduleCodePrefixRule {
    fn check(&self, item: &BatchedGrantSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantScheduleRequiredNameRule {
    pub message: String,
}

impl BatchedGrantScheduleRule for BatchedGrantScheduleRequiredNameRule {
    fn check(&self, item: &BatchedGrantSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantScheduleNameLengthRule {
    pub max_length: i64,
}

impl BatchedGrantScheduleRule for BatchedGrantScheduleNameLengthRule {
    fn check(&self, item: &BatchedGrantSchedule) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched grant schedule.
#[derive(Debug, Clone)]
pub struct BatchedGrantScheduleValidator {
    pub rules: Vec<Rc<dyn BatchedGrantScheduleRule>>,
}

impl BatchedGrantScheduleValidator {
    pub fn validate(&self, item: &BatchedGrantSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedGrantSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedGrantSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_grant_schedule_validator() -> BatchedGrantScheduleValidator {
    BatchedGrantScheduleValidator {
        rules: vec![
            Rc::new(BatchedGrantScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn BatchedGrantScheduleRule>,
            Rc::new(BatchedGrantScheduleRequiredNameRule {
                message: "batched grant schedule needs this field".to_string(),
            }),
            Rc::new(BatchedGrantScheduleNameLengthRule { max_length: 64 }),
        ],
    }
}
