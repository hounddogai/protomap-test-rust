use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit schedule must pass. Check returns its problems, or none.
pub trait LimitScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LimitSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitScheduleLabelLengthRule {
    pub max_length: i64,
}

impl LimitScheduleRule for LimitScheduleLabelLengthRule {
    fn check(&self, item: &LimitSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitScheduleRequiredLabelRule {
    pub message: String,
}

impl LimitScheduleRule for LimitScheduleRequiredLabelRule {
    fn check(&self, item: &LimitSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitScheduleIssuerLengthRule {
    pub max_length: i64,
}

impl LimitScheduleRule for LimitScheduleIssuerLengthRule {
    fn check(&self, item: &LimitSchedule) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitScheduleRequiredCodeRule {
    pub message: String,
}

impl LimitScheduleRule for LimitScheduleRequiredCodeRule {
    fn check(&self, item: &LimitSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit schedule.
#[derive(Debug, Clone)]
pub struct LimitScheduleValidator {
    pub rules: Vec<Rc<dyn LimitScheduleRule>>,
}

impl LimitScheduleValidator {
    pub fn validate(&self, item: &LimitSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_schedule_validator() -> LimitScheduleValidator {
    LimitScheduleValidator {
        rules: vec![
            Rc::new(LimitScheduleLabelLengthRule { max_length: 120 }) as Rc<dyn LimitScheduleRule>,
            Rc::new(LimitScheduleRequiredLabelRule {
                message: "limit schedule needs this field".to_string(),
            }),
            Rc::new(LimitScheduleIssuerLengthRule { max_length: 64 }),
            Rc::new(LimitScheduleRequiredCodeRule {
                message: "limit schedule needs an entry".to_string(),
            }),
        ],
    }
}
