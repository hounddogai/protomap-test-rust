use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction schedule must pass. Check returns its problems, or none.
pub trait RedactionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RedactionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleUnitLengthRule {
    pub max_length: i64,
}

impl RedactionScheduleRule for RedactionScheduleUnitLengthRule {
    fn check(&self, item: &RedactionSchedule) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleCodePrefixRule {
    pub prefix: String,
}

impl RedactionScheduleRule for RedactionScheduleCodePrefixRule {
    fn check(&self, item: &RedactionSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionScheduleRule for RedactionScheduleScoreRangeRule {
    fn check(&self, item: &RedactionSchedule) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleRequiredCipherRule {
    pub message: String,
}

impl RedactionScheduleRule for RedactionScheduleRequiredCipherRule {
    fn check(&self, item: &RedactionSchedule) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RedactionScheduleRule for RedactionScheduleCodeLengthRule {
    fn check(&self, item: &RedactionSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScheduleCipherLengthRule {
    pub max_length: i64,
}

impl RedactionScheduleRule for RedactionScheduleCipherLengthRule {
    fn check(&self, item: &RedactionSchedule) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction schedule.
#[derive(Debug, Clone)]
pub struct RedactionScheduleValidator {
    pub rules: Vec<Rc<dyn RedactionScheduleRule>>,
}

impl RedactionScheduleValidator {
    pub fn validate(&self, item: &RedactionSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_schedule_validator() -> RedactionScheduleValidator {
    RedactionScheduleValidator {
        rules: vec![
            Rc::new(RedactionScheduleUnitLengthRule { max_length: 32 }) as Rc<dyn RedactionScheduleRule>,
            Rc::new(RedactionScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(RedactionScheduleScoreRangeRule { min: 0.6, max: 4137.8 }),
            Rc::new(RedactionScheduleRequiredCipherRule {
                message: "redaction schedule needs an entry".to_string(),
            }),
            Rc::new(RedactionScheduleCodeLengthRule { max_length: 64 }),
            Rc::new(RedactionScheduleCipherLengthRule { max_length: 86400 }),
        ],
    }
}
