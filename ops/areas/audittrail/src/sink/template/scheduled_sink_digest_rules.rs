use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled sink digest must pass. Check returns its problems, or none.
pub trait ScheduledSinkDigestRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSinkDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkDigestRequiredLabelRule {
    pub message: String,
}

impl ScheduledSinkDigestRule for ScheduledSinkDigestRequiredLabelRule {
    fn check(&self, item: &ScheduledSinkDigest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkDigestCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledSinkDigestRule for ScheduledSinkDigestCodeLengthRule {
    fn check(&self, item: &ScheduledSinkDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkDigestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSinkDigestRule for ScheduledSinkDigestGraceDaysRangeRule {
    fn check(&self, item: &ScheduledSinkDigest) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled sink digest.
#[derive(Debug, Clone)]
pub struct ScheduledSinkDigestValidator {
    pub rules: Vec<Rc<dyn ScheduledSinkDigestRule>>,
}

impl ScheduledSinkDigestValidator {
    pub fn validate(&self, item: &ScheduledSinkDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSinkDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSinkDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_sink_digest_validator() -> ScheduledSinkDigestValidator {
    ScheduledSinkDigestValidator {
        rules: vec![
            Rc::new(ScheduledSinkDigestRequiredLabelRule {
                message: "scheduled sink digest needs an entry".to_string(),
            }) as Rc<dyn ScheduledSinkDigestRule>,
            Rc::new(ScheduledSinkDigestCodeLengthRule { max_length: 120 }),
            Rc::new(ScheduledSinkDigestGraceDaysRangeRule { min: 255, max: 3600 }),
        ],
    }
}
