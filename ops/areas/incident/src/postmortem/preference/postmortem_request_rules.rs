use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem request must pass. Check returns its problems, or none.
pub trait PostmortemRequestRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRequestLabelLengthRule {
    pub max_length: i64,
}

impl PostmortemRequestRule for PostmortemRequestLabelLengthRule {
    fn check(&self, item: &PostmortemRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRequestQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemRequestRule for PostmortemRequestQuotaUnitsRangeRule {
    fn check(&self, item: &PostmortemRequest) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRequestRequiredLabelRule {
    pub message: String,
}

impl PostmortemRequestRule for PostmortemRequestRequiredLabelRule {
    fn check(&self, item: &PostmortemRequest) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRequestRequiredCodeRule {
    pub message: String,
}

impl PostmortemRequestRule for PostmortemRequestRequiredCodeRule {
    fn check(&self, item: &PostmortemRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem request.
#[derive(Debug, Clone)]
pub struct PostmortemRequestValidator {
    pub rules: Vec<Rc<dyn PostmortemRequestRule>>,
}

impl PostmortemRequestValidator {
    pub fn validate(&self, item: &PostmortemRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_request_validator() -> PostmortemRequestValidator {
    PostmortemRequestValidator {
        rules: vec![
            Rc::new(PostmortemRequestLabelLengthRule { max_length: 3600 }) as Rc<dyn PostmortemRequestRule>,
            Rc::new(PostmortemRequestQuotaUnitsRangeRule { min: 80, max: 1000 }),
            Rc::new(PostmortemRequestRequiredLabelRule {
                message: "postmortem request needs this field".to_string(),
            }),
            Rc::new(PostmortemRequestRequiredCodeRule {
                message: "postmortem request needs this field".to_string(),
            }),
        ],
    }
}
