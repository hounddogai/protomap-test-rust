use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict segment window must pass. Check returns its problems, or none.
pub trait StrictSegmentWindowRule: std::fmt::Debug {
    fn check(&self, item: &StrictSegmentWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowRequiredAudienceRule {
    pub message: String,
}

impl StrictSegmentWindowRule for StrictSegmentWindowRequiredAudienceRule {
    fn check(&self, item: &StrictSegmentWindow) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowCodeLengthRule {
    pub max_length: i64,
}

impl StrictSegmentWindowRule for StrictSegmentWindowCodeLengthRule {
    fn check(&self, item: &StrictSegmentWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowNameLengthRule {
    pub max_length: i64,
}

impl StrictSegmentWindowRule for StrictSegmentWindowNameLengthRule {
    fn check(&self, item: &StrictSegmentWindow) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentWindowRequiredNameRule {
    pub message: String,
}

impl StrictSegmentWindowRule for StrictSegmentWindowRequiredNameRule {
    fn check(&self, item: &StrictSegmentWindow) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict segment window.
#[derive(Debug, Clone)]
pub struct StrictSegmentWindowValidator {
    pub rules: Vec<Rc<dyn StrictSegmentWindowRule>>,
}

impl StrictSegmentWindowValidator {
    pub fn validate(&self, item: &StrictSegmentWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictSegmentWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSegmentWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_segment_window_validator() -> StrictSegmentWindowValidator {
    StrictSegmentWindowValidator {
        rules: vec![
            Rc::new(StrictSegmentWindowRequiredAudienceRule {
                message: "strict segment window needs this field".to_string(),
            }) as Rc<dyn StrictSegmentWindowRule>,
            Rc::new(StrictSegmentWindowCodeLengthRule { max_length: 16 }),
            Rc::new(StrictSegmentWindowNameLengthRule { max_length: 32 }),
            Rc::new(StrictSegmentWindowRequiredNameRule {
                message: "strict segment window needs an entry".to_string(),
            }),
        ],
    }
}
