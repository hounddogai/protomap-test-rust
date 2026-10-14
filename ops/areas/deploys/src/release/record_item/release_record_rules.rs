use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release record must pass. Check returns its problems, or none.
pub trait ReleaseRecordRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRecordRequiredPhoneRule {
    pub message: String,
}

impl ReleaseRecordRule for ReleaseRecordRequiredPhoneRule {
    fn check(&self, item: &ReleaseRecord) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRecordTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseRecordRule for ReleaseRecordTtlSecondsRangeRule {
    fn check(&self, item: &ReleaseRecord) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRecordPrincipalLengthRule {
    pub max_length: i64,
}

impl ReleaseRecordRule for ReleaseRecordPrincipalLengthRule {
    fn check(&self, item: &ReleaseRecord) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a release record.
#[derive(Debug, Clone)]
pub struct ReleaseRecordValidator {
    pub rules: Vec<Rc<dyn ReleaseRecordRule>>,
}

impl ReleaseRecordValidator {
    pub fn validate(&self, item: &ReleaseRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_record_validator() -> ReleaseRecordValidator {
    ReleaseRecordValidator {
        rules: vec![
            Rc::new(ReleaseRecordRequiredPhoneRule {
                message: "release record needs a value".to_string(),
            }) as Rc<dyn ReleaseRecordRule>,
            Rc::new(ReleaseRecordTtlSecondsRangeRule { min: 16, max: 1000 }),
            Rc::new(ReleaseRecordPrincipalLengthRule { max_length: 1000 }),
        ],
    }
}
