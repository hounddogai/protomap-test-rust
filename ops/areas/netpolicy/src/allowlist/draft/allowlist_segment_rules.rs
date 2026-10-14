use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist segment must pass. Check returns its problems, or none.
pub trait AllowlistSegmentRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSegmentContactEmailLengthRule {
    pub max_length: i64,
}

impl AllowlistSegmentRule for AllowlistSegmentContactEmailLengthRule {
    fn check(&self, item: &AllowlistSegment) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSegmentExternalRefLengthRule {
    pub max_length: i64,
}

impl AllowlistSegmentRule for AllowlistSegmentExternalRefLengthRule {
    fn check(&self, item: &AllowlistSegment) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSegmentAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistSegmentRule for AllowlistSegmentAttemptsRangeRule {
    fn check(&self, item: &AllowlistSegment) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSegmentRequiredContactEmailRule {
    pub message: String,
}

impl AllowlistSegmentRule for AllowlistSegmentRequiredContactEmailRule {
    fn check(&self, item: &AllowlistSegment) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist segment.
#[derive(Debug, Clone)]
pub struct AllowlistSegmentValidator {
    pub rules: Vec<Rc<dyn AllowlistSegmentRule>>,
}

impl AllowlistSegmentValidator {
    pub fn validate(&self, item: &AllowlistSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_segment_validator() -> AllowlistSegmentValidator {
    AllowlistSegmentValidator {
        rules: vec![
            Rc::new(AllowlistSegmentContactEmailLengthRule { max_length: 32 }) as Rc<dyn AllowlistSegmentRule>,
            Rc::new(AllowlistSegmentExternalRefLengthRule { max_length: 255 }),
            Rc::new(AllowlistSegmentAttemptsRangeRule { min: 120, max: 3600 }),
            Rc::new(AllowlistSegmentRequiredContactEmailRule {
                message: "allowlist segment needs this field".to_string(),
            }),
        ],
    }
}
