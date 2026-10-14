use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual evidence draft must pass. Check returns its problems, or none.
pub trait ManualEvidenceDraftRule: std::fmt::Debug {
    fn check(&self, item: &ManualEvidenceDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceDraftLocaleLengthRule {
    pub max_length: i64,
}

impl ManualEvidenceDraftRule for ManualEvidenceDraftLocaleLengthRule {
    fn check(&self, item: &ManualEvidenceDraft) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceDraftCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualEvidenceDraftRule for ManualEvidenceDraftCapacityRangeRule {
    fn check(&self, item: &ManualEvidenceDraft) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceDraftRequiredLocaleRule {
    pub message: String,
}

impl ManualEvidenceDraftRule for ManualEvidenceDraftRequiredLocaleRule {
    fn check(&self, item: &ManualEvidenceDraft) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceDraftVisibleGuardRule {
    pub limit: i64,
}

impl ManualEvidenceDraftRule for ManualEvidenceDraftVisibleGuardRule {
    fn check(&self, item: &ManualEvidenceDraft) -> Vec<String> {
        if item.visible && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while visible",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceDraftLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualEvidenceDraftRule for ManualEvidenceDraftLimitRangeRule {
    fn check(&self, item: &ManualEvidenceDraft) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual evidence draft.
#[derive(Debug, Clone)]
pub struct ManualEvidenceDraftValidator {
    pub rules: Vec<Rc<dyn ManualEvidenceDraftRule>>,
}

impl ManualEvidenceDraftValidator {
    pub fn validate(&self, item: &ManualEvidenceDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualEvidenceDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualEvidenceDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_evidence_draft_validator() -> ManualEvidenceDraftValidator {
    ManualEvidenceDraftValidator {
        rules: vec![
            Rc::new(ManualEvidenceDraftLocaleLengthRule { max_length: 120 }) as Rc<dyn ManualEvidenceDraftRule>,
            Rc::new(ManualEvidenceDraftCapacityRangeRule { min: 32, max: 86400 }),
            Rc::new(ManualEvidenceDraftRequiredLocaleRule {
                message: "manual evidence draft needs an entry".to_string(),
            }),
            Rc::new(ManualEvidenceDraftVisibleGuardRule { limit: 32 }),
            Rc::new(ManualEvidenceDraftLimitRangeRule { min: 80, max: 120 }),
        ],
    }
}
