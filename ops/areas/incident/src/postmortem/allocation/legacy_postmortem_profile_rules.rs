use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy postmortem profile must pass. Check returns its problems, or none.
pub trait LegacyPostmortemProfileRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPostmortemProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemProfileCodePrefixRule {
    pub prefix: String,
}

impl LegacyPostmortemProfileRule for LegacyPostmortemProfileCodePrefixRule {
    fn check(&self, item: &LegacyPostmortemProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemProfileRequiredCodeRule {
    pub message: String,
}

impl LegacyPostmortemProfileRule for LegacyPostmortemProfileRequiredCodeRule {
    fn check(&self, item: &LegacyPostmortemProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemProfileScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyPostmortemProfileRule for LegacyPostmortemProfileScoreRangeRule {
    fn check(&self, item: &LegacyPostmortemProfile) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPostmortemProfileSegmentLengthRule {
    pub max_length: i64,
}

impl LegacyPostmortemProfileRule for LegacyPostmortemProfileSegmentLengthRule {
    fn check(&self, item: &LegacyPostmortemProfile) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy postmortem profile.
#[derive(Debug, Clone)]
pub struct LegacyPostmortemProfileValidator {
    pub rules: Vec<Rc<dyn LegacyPostmortemProfileRule>>,
}

impl LegacyPostmortemProfileValidator {
    pub fn validate(&self, item: &LegacyPostmortemProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyPostmortemProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPostmortemProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_postmortem_profile_validator() -> LegacyPostmortemProfileValidator {
    LegacyPostmortemProfileValidator {
        rules: vec![
            Rc::new(LegacyPostmortemProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn LegacyPostmortemProfileRule>,
            Rc::new(LegacyPostmortemProfileRequiredCodeRule {
                message: "legacy postmortem profile needs this field".to_string(),
            }),
            Rc::new(LegacyPostmortemProfileScoreRangeRule { min: 8.7, max: 3189.1 }),
            Rc::new(LegacyPostmortemProfileSegmentLengthRule { max_length: 32 }),
        ],
    }
}
