use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled evidence tier must pass. Check returns its problems, or none.
pub trait ScheduledEvidenceTierRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledEvidenceTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierRequiredCodeRule {
    pub message: String,
}

impl ScheduledEvidenceTierRule for ScheduledEvidenceTierRequiredCodeRule {
    fn check(&self, item: &ScheduledEvidenceTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledEvidenceTierRule for ScheduledEvidenceTierCodeLengthRule {
    fn check(&self, item: &ScheduledEvidenceTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledEvidenceTierRule for ScheduledEvidenceTierLabelLengthRule {
    fn check(&self, item: &ScheduledEvidenceTier) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledEvidenceTierRule for ScheduledEvidenceTierEndsAtRangeRule {
    fn check(&self, item: &ScheduledEvidenceTier) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledEvidenceTierRule for ScheduledEvidenceTierRetriesRangeRule {
    fn check(&self, item: &ScheduledEvidenceTier) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a scheduled evidence tier.
#[derive(Debug, Clone)]
pub struct ScheduledEvidenceTierValidator {
    pub rules: Vec<Rc<dyn ScheduledEvidenceTierRule>>,
}

impl ScheduledEvidenceTierValidator {
    pub fn validate(&self, item: &ScheduledEvidenceTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledEvidenceTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledEvidenceTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_evidence_tier_validator() -> ScheduledEvidenceTierValidator {
    ScheduledEvidenceTierValidator {
        rules: vec![
            Rc::new(ScheduledEvidenceTierRequiredCodeRule {
                message: "scheduled evidence tier needs this field".to_string(),
            }) as Rc<dyn ScheduledEvidenceTierRule>,
            Rc::new(ScheduledEvidenceTierCodeLengthRule { max_length: 3600 }),
            Rc::new(ScheduledEvidenceTierLabelLengthRule { max_length: 3600 }),
            Rc::new(ScheduledEvidenceTierEndsAtRangeRule { min: 120, max: 255 }),
            Rc::new(ScheduledEvidenceTierRetriesRangeRule { min: 16, max: 255 }),
        ],
    }
}
