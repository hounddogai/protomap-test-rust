use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered rollout segment must pass. Check returns its problems, or none.
pub trait TieredRolloutSegmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredRolloutSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentAudienceLengthRule {
    pub max_length: i64,
}

impl TieredRolloutSegmentRule for TieredRolloutSegmentAudienceLengthRule {
    fn check(&self, item: &TieredRolloutSegment) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentRequiredLabelRule {
    pub message: String,
}

impl TieredRolloutSegmentRule for TieredRolloutSegmentRequiredLabelRule {
    fn check(&self, item: &TieredRolloutSegment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentArchivedGuardRule {
    pub limit: i64,
}

impl TieredRolloutSegmentRule for TieredRolloutSegmentArchivedGuardRule {
    fn check(&self, item: &TieredRolloutSegment) -> Vec<String> {
        if item.archived && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while archived",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentCodePrefixRule {
    pub prefix: String,
}

impl TieredRolloutSegmentRule for TieredRolloutSegmentCodePrefixRule {
    fn check(&self, item: &TieredRolloutSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutSegmentCodeLengthRule {
    pub max_length: i64,
}

impl TieredRolloutSegmentRule for TieredRolloutSegmentCodeLengthRule {
    fn check(&self, item: &TieredRolloutSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered rollout segment.
#[derive(Debug, Clone)]
pub struct TieredRolloutSegmentValidator {
    pub rules: Vec<Rc<dyn TieredRolloutSegmentRule>>,
}

impl TieredRolloutSegmentValidator {
    pub fn validate(&self, item: &TieredRolloutSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRolloutSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRolloutSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_rollout_segment_validator() -> TieredRolloutSegmentValidator {
    TieredRolloutSegmentValidator {
        rules: vec![
            Rc::new(TieredRolloutSegmentAudienceLengthRule { max_length: 86400 }) as Rc<dyn TieredRolloutSegmentRule>,
            Rc::new(TieredRolloutSegmentRequiredLabelRule {
                message: "tiered rollout segment needs this field".to_string(),
            }),
            Rc::new(TieredRolloutSegmentArchivedGuardRule { limit: 86400 }),
            Rc::new(TieredRolloutSegmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(TieredRolloutSegmentCodeLengthRule { max_length: 3600 }),
        ],
    }
}
