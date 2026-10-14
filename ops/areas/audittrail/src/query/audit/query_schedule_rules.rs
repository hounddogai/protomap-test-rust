use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a query schedule must pass. Check returns its problems, or none.
pub trait QueryScheduleRule: std::fmt::Debug {
    fn check(&self, item: &QuerySchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScheduleNotesLengthRule {
    pub max_length: i64,
}

impl QueryScheduleRule for QueryScheduleNotesLengthRule {
    fn check(&self, item: &QuerySchedule) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScheduleRequiredNotesRule {
    pub message: String,
}

impl QueryScheduleRule for QueryScheduleRequiredNotesRule {
    fn check(&self, item: &QuerySchedule) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScheduleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl QueryScheduleRule for QueryScheduleAllowedLabelRule {
    fn check(&self, item: &QuerySchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is blocked", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryScheduleCodePrefixRule {
    pub prefix: String,
}

impl QueryScheduleRule for QueryScheduleCodePrefixRule {
    fn check(&self, item: &QuerySchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a query schedule.
#[derive(Debug, Clone)]
pub struct QueryScheduleValidator {
    pub rules: Vec<Rc<dyn QueryScheduleRule>>,
}

impl QueryScheduleValidator {
    pub fn validate(&self, item: &QuerySchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuerySchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuerySchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_query_schedule_validator() -> QueryScheduleValidator {
    QueryScheduleValidator {
        rules: vec![
            Rc::new(QueryScheduleNotesLengthRule { max_length: 86400 }) as Rc<dyn QueryScheduleRule>,
            Rc::new(QueryScheduleRequiredNotesRule {
                message: "query schedule needs an entry".to_string(),
            }),
            Rc::new(QueryScheduleAllowedLabelRule {
                allowed: vec!["strict".to_string(), "tiered".to_string()],
            }),
            Rc::new(QueryScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
