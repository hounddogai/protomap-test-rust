use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global limit draft must pass. Check returns its problems, or none.
pub trait GlobalLimitDraftRule: std::fmt::Debug {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftCodePrefixRule {
    pub prefix: String,
}

impl GlobalLimitDraftRule for GlobalLimitDraftCodePrefixRule {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftRequiredTimezoneRule {
    pub message: String,
}

impl GlobalLimitDraftRule for GlobalLimitDraftRequiredTimezoneRule {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftRequiredTitleRule {
    pub message: String,
}

impl GlobalLimitDraftRule for GlobalLimitDraftRequiredTitleRule {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl GlobalLimitDraftRule for GlobalLimitDraftAllowedTimezoneRule {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is blocked", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftTimezoneLengthRule {
    pub max_length: i64,
}

impl GlobalLimitDraftRule for GlobalLimitDraftTimezoneLengthRule {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitDraftApprovedGuardRule {
    pub limit: f64,
}

impl GlobalLimitDraftRule for GlobalLimitDraftApprovedGuardRule {
    fn check(&self, item: &GlobalLimitDraft) -> Vec<String> {
        if item.approved && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while approved",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global limit draft.
#[derive(Debug, Clone)]
pub struct GlobalLimitDraftValidator {
    pub rules: Vec<Rc<dyn GlobalLimitDraftRule>>,
}

impl GlobalLimitDraftValidator {
    pub fn validate(&self, item: &GlobalLimitDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalLimitDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalLimitDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_limit_draft_validator() -> GlobalLimitDraftValidator {
    GlobalLimitDraftValidator {
        rules: vec![
            Rc::new(GlobalLimitDraftCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn GlobalLimitDraftRule>,
            Rc::new(GlobalLimitDraftRequiredTimezoneRule {
                message: "global limit draft needs this field".to_string(),
            }),
            Rc::new(GlobalLimitDraftRequiredTitleRule {
                message: "global limit draft needs an entry".to_string(),
            }),
            Rc::new(GlobalLimitDraftAllowedTimezoneRule {
                allowed: vec!["rolling".to_string(), "adaptive".to_string(), "legacy".to_string()],
            }),
            Rc::new(GlobalLimitDraftTimezoneLengthRule { max_length: 1000 }),
            Rc::new(GlobalLimitDraftApprovedGuardRule { limit: 1.4 }),
        ],
    }
}
