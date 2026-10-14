use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive plan schedule must pass. Check returns its problems, or none.
pub trait AdaptivePlanScheduleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePlanSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanScheduleRequiredCodeRule {
    pub message: String,
}

impl AdaptivePlanScheduleRule for AdaptivePlanScheduleRequiredCodeRule {
    fn check(&self, item: &AdaptivePlanSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanScheduleRequiredHostRule {
    pub message: String,
}

impl AdaptivePlanScheduleRule for AdaptivePlanScheduleRequiredHostRule {
    fn check(&self, item: &AdaptivePlanSchedule) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanScheduleFingerprintLengthRule {
    pub max_length: i64,
}

impl AdaptivePlanScheduleRule for AdaptivePlanScheduleFingerprintLengthRule {
    fn check(&self, item: &AdaptivePlanSchedule) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanScheduleLabelLengthRule {
    pub max_length: i64,
}

impl AdaptivePlanScheduleRule for AdaptivePlanScheduleLabelLengthRule {
    fn check(&self, item: &AdaptivePlanSchedule) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive plan schedule.
#[derive(Debug, Clone)]
pub struct AdaptivePlanScheduleValidator {
    pub rules: Vec<Rc<dyn AdaptivePlanScheduleRule>>,
}

impl AdaptivePlanScheduleValidator {
    pub fn validate(&self, item: &AdaptivePlanSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptivePlanSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePlanSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_plan_schedule_validator() -> AdaptivePlanScheduleValidator {
    AdaptivePlanScheduleValidator {
        rules: vec![
            Rc::new(AdaptivePlanScheduleRequiredCodeRule {
                message: "adaptive plan schedule needs this field".to_string(),
            }) as Rc<dyn AdaptivePlanScheduleRule>,
            Rc::new(AdaptivePlanScheduleRequiredHostRule {
                message: "adaptive plan schedule needs this field".to_string(),
            }),
            Rc::new(AdaptivePlanScheduleFingerprintLengthRule { max_length: 64 }),
            Rc::new(AdaptivePlanScheduleLabelLengthRule { max_length: 120 }),
        ],
    }
}
