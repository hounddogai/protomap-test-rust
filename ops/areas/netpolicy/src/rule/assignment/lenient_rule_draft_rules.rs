use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient rule draft must pass. Check returns its problems, or none.
pub trait LenientRuleDraftRule: std::fmt::Debug {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraftAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl LenientRuleDraftRule for LenientRuleDraftAllowedUnitRule {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is above maximum", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraftPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientRuleDraftRule for LenientRuleDraftPriorityRangeRule {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraftCodeLengthRule {
    pub max_length: i64,
}

impl LenientRuleDraftRule for LenientRuleDraftCodeLengthRule {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraftRequiredOwnerIdRule {
    pub message: String,
}

impl LenientRuleDraftRule for LenientRuleDraftRequiredOwnerIdRule {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraftTitleLengthRule {
    pub max_length: i64,
}

impl LenientRuleDraftRule for LenientRuleDraftTitleLengthRule {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleDraftRequiredTitleRule {
    pub message: String,
}

impl LenientRuleDraftRule for LenientRuleDraftRequiredTitleRule {
    fn check(&self, item: &LenientRuleDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient rule draft.
#[derive(Debug, Clone)]
pub struct LenientRuleDraftValidator {
    pub rules: Vec<Rc<dyn LenientRuleDraftRule>>,
}

impl LenientRuleDraftValidator {
    pub fn validate(&self, item: &LenientRuleDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientRuleDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRuleDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_rule_draft_validator() -> LenientRuleDraftValidator {
    LenientRuleDraftValidator {
        rules: vec![
            Rc::new(LenientRuleDraftAllowedUnitRule { allowed: Vec::new() }) as Rc<dyn LenientRuleDraftRule>,
            Rc::new(LenientRuleDraftPriorityRangeRule { min: 16, max: 80 }),
            Rc::new(LenientRuleDraftCodeLengthRule { max_length: 64 }),
            Rc::new(LenientRuleDraftRequiredOwnerIdRule {
                message: "lenient rule draft needs this field".to_string(),
            }),
            Rc::new(LenientRuleDraftTitleLengthRule { max_length: 16 }),
            Rc::new(LenientRuleDraftRequiredTitleRule {
                message: "lenient rule draft needs an entry".to_string(),
            }),
        ],
    }
}
