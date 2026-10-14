use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem draft must pass. Check returns its problems, or none.
pub trait PostmortemDraftRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PostmortemDraftRule for PostmortemDraftAllowedTitleRule {
    fn check(&self, item: &PostmortemDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemDraftRule for PostmortemDraftCapacityRangeRule {
    fn check(&self, item: &PostmortemDraft) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftTitleLengthRule {
    pub max_length: i64,
}

impl PostmortemDraftRule for PostmortemDraftTitleLengthRule {
    fn check(&self, item: &PostmortemDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemDraftRule for PostmortemDraftRetriesRangeRule {
    fn check(&self, item: &PostmortemDraft) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftCodePrefixRule {
    pub prefix: String,
}

impl PostmortemDraftRule for PostmortemDraftCodePrefixRule {
    fn check(&self, item: &PostmortemDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem draft.
#[derive(Debug, Clone)]
pub struct PostmortemDraftValidator {
    pub rules: Vec<Rc<dyn PostmortemDraftRule>>,
}

impl PostmortemDraftValidator {
    pub fn validate(&self, item: &PostmortemDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PostmortemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_draft_validator() -> PostmortemDraftValidator {
    PostmortemDraftValidator {
        rules: vec![
            Rc::new(PostmortemDraftAllowedTitleRule { allowed: Vec::new() }) as Rc<dyn PostmortemDraftRule>,
            Rc::new(PostmortemDraftCapacityRangeRule { min: 86400, max: 86400 }),
            Rc::new(PostmortemDraftTitleLengthRule { max_length: 64 }),
            Rc::new(PostmortemDraftRetriesRangeRule { min: 32, max: 1000 }),
            Rc::new(PostmortemDraftCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
