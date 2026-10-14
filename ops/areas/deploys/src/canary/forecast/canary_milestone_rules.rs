use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary milestone must pass. Check returns its problems, or none.
pub trait CanaryMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &CanaryMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMilestoneRequiredCodeRule {
    pub message: String,
}

impl CanaryMilestoneRule for CanaryMilestoneRequiredCodeRule {
    fn check(&self, item: &CanaryMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMilestoneRequiredFingerprintRule {
    pub message: String,
}

impl CanaryMilestoneRule for CanaryMilestoneRequiredFingerprintRule {
    fn check(&self, item: &CanaryMilestone) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMilestoneFingerprintLengthRule {
    pub max_length: i64,
}

impl CanaryMilestoneRule for CanaryMilestoneFingerprintLengthRule {
    fn check(&self, item: &CanaryMilestone) -> Vec<String> {
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
pub struct CanaryMilestoneMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryMilestoneRule for CanaryMilestoneMinimumRangeRule {
    fn check(&self, item: &CanaryMilestone) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryMilestoneRequiredSummaryRule {
    pub message: String,
}

impl CanaryMilestoneRule for CanaryMilestoneRequiredSummaryRule {
    fn check(&self, item: &CanaryMilestone) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary milestone.
#[derive(Debug, Clone)]
pub struct CanaryMilestoneValidator {
    pub rules: Vec<Rc<dyn CanaryMilestoneRule>>,
}

impl CanaryMilestoneValidator {
    pub fn validate(&self, item: &CanaryMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_milestone_validator() -> CanaryMilestoneValidator {
    CanaryMilestoneValidator {
        rules: vec![
            Rc::new(CanaryMilestoneRequiredCodeRule {
                message: "canary milestone needs this field".to_string(),
            }) as Rc<dyn CanaryMilestoneRule>,
            Rc::new(CanaryMilestoneRequiredFingerprintRule {
                message: "canary milestone needs a value".to_string(),
            }),
            Rc::new(CanaryMilestoneFingerprintLengthRule { max_length: 1000 }),
            Rc::new(CanaryMilestoneMinimumRangeRule { min: 5.3, max: 614.3 }),
            Rc::new(CanaryMilestoneRequiredSummaryRule {
                message: "canary milestone needs this field".to_string(),
            }),
        ],
    }
}
