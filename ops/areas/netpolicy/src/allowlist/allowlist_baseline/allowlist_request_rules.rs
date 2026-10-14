use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist request must pass. Check returns its problems, or none.
pub trait AllowlistRequestRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequestAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AllowlistRequestRule for AllowlistRequestAllowedLabelRule {
    fn check(&self, item: &AllowlistRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequestRequiredNotesRule {
    pub message: String,
}

impl AllowlistRequestRule for AllowlistRequestRequiredNotesRule {
    fn check(&self, item: &AllowlistRequest) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequestNamespaceLengthRule {
    pub max_length: i64,
}

impl AllowlistRequestRule for AllowlistRequestNamespaceLengthRule {
    fn check(&self, item: &AllowlistRequest) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequestLabelLengthRule {
    pub max_length: i64,
}

impl AllowlistRequestRule for AllowlistRequestLabelLengthRule {
    fn check(&self, item: &AllowlistRequest) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequestRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistRequestRule for AllowlistRequestRatioRangeRule {
    fn check(&self, item: &AllowlistRequest) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist request.
#[derive(Debug, Clone)]
pub struct AllowlistRequestValidator {
    pub rules: Vec<Rc<dyn AllowlistRequestRule>>,
}

impl AllowlistRequestValidator {
    pub fn validate(&self, item: &AllowlistRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_request_validator() -> AllowlistRequestValidator {
    AllowlistRequestValidator {
        rules: vec![
            Rc::new(AllowlistRequestAllowedLabelRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn AllowlistRequestRule>,
            Rc::new(AllowlistRequestRequiredNotesRule {
                message: "allowlist request needs a value".to_string(),
            }),
            Rc::new(AllowlistRequestNamespaceLengthRule { max_length: 1000 }),
            Rc::new(AllowlistRequestLabelLengthRule { max_length: 120 }),
            Rc::new(AllowlistRequestRatioRangeRule { min: 7.1, max: 167.7 }),
        ],
    }
}
