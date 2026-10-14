use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment score must pass. Check returns its problems, or none.
pub trait SegmentScoreRule: std::fmt::Debug {
    fn check(&self, item: &SegmentScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScoreRequiredLabelRule {
    pub message: String,
}

impl SegmentScoreRule for SegmentScoreRequiredLabelRule {
    fn check(&self, item: &SegmentScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScoreCodePrefixRule {
    pub prefix: String,
}

impl SegmentScoreRule for SegmentScoreCodePrefixRule {
    fn check(&self, item: &SegmentScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SegmentScoreRule for SegmentScoreAllowedCodeRule {
    fn check(&self, item: &SegmentScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is missing", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment score.
#[derive(Debug, Clone)]
pub struct SegmentScoreValidator {
    pub rules: Vec<Rc<dyn SegmentScoreRule>>,
}

impl SegmentScoreValidator {
    pub fn validate(&self, item: &SegmentScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_score_validator() -> SegmentScoreValidator {
    SegmentScoreValidator {
        rules: vec![
            Rc::new(SegmentScoreRequiredLabelRule {
                message: "segment score needs a value".to_string(),
            }) as Rc<dyn SegmentScoreRule>,
            Rc::new(SegmentScoreCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SegmentScoreAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}
