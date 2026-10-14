use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional page draft must pass. Check returns its problems, or none.
pub trait RegionalPageDraftRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPageDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftCodeLengthRule {
    pub max_length: i64,
}

impl RegionalPageDraftRule for RegionalPageDraftCodeLengthRule {
    fn check(&self, item: &RegionalPageDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftRequiredTitleRule {
    pub message: String,
}

impl RegionalPageDraftRule for RegionalPageDraftRequiredTitleRule {
    fn check(&self, item: &RegionalPageDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftPrincipalLengthRule {
    pub max_length: i64,
}

impl RegionalPageDraftRule for RegionalPageDraftPrincipalLengthRule {
    fn check(&self, item: &RegionalPageDraft) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftRequiredCodeRule {
    pub message: String,
}

impl RegionalPageDraftRule for RegionalPageDraftRequiredCodeRule {
    fn check(&self, item: &RegionalPageDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RegionalPageDraftRule for RegionalPageDraftTagsLimitRule {
    fn check(&self, item: &RegionalPageDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

/// Runs every rule of a regional page draft.
#[derive(Debug, Clone)]
pub struct RegionalPageDraftValidator {
    pub rules: Vec<Rc<dyn RegionalPageDraftRule>>,
}

impl RegionalPageDraftValidator {
    pub fn validate(&self, item: &RegionalPageDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPageDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPageDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_page_draft_validator() -> RegionalPageDraftValidator {
    RegionalPageDraftValidator {
        rules: vec![
            Rc::new(RegionalPageDraftCodeLengthRule { max_length: 1000 }) as Rc<dyn RegionalPageDraftRule>,
            Rc::new(RegionalPageDraftRequiredTitleRule {
                message: "regional page draft needs a value".to_string(),
            }),
            Rc::new(RegionalPageDraftPrincipalLengthRule { max_length: 120 }),
            Rc::new(RegionalPageDraftRequiredCodeRule {
                message: "regional page draft needs this field".to_string(),
            }),
            Rc::new(RegionalPageDraftTagsLimitRule {
                max_count: 1000,
                banned: vec!["adaptive".to_string()],
            }),
        ],
    }
}
