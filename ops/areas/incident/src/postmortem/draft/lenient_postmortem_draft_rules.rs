use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient postmortem draft must pass. Check returns its problems, or none.
pub trait LenientPostmortemDraftRule: std::fmt::Debug {
    fn check(&self, item: &LenientPostmortemDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemDraftRequiredCodeRule {
    pub message: String,
}

impl LenientPostmortemDraftRule for LenientPostmortemDraftRequiredCodeRule {
    fn check(&self, item: &LenientPostmortemDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemDraftScopeLengthRule {
    pub max_length: i64,
}

impl LenientPostmortemDraftRule for LenientPostmortemDraftScopeLengthRule {
    fn check(&self, item: &LenientPostmortemDraft) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemDraftWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPostmortemDraftRule for LenientPostmortemDraftWindowDaysRangeRule {
    fn check(&self, item: &LenientPostmortemDraft) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient postmortem draft.
#[derive(Debug, Clone)]
pub struct LenientPostmortemDraftValidator {
    pub rules: Vec<Rc<dyn LenientPostmortemDraftRule>>,
}

impl LenientPostmortemDraftValidator {
    pub fn validate(&self, item: &LenientPostmortemDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPostmortemDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPostmortemDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_postmortem_draft_validator() -> LenientPostmortemDraftValidator {
    LenientPostmortemDraftValidator {
        rules: vec![
            Rc::new(LenientPostmortemDraftRequiredCodeRule {
                message: "lenient postmortem draft needs an entry".to_string(),
            }) as Rc<dyn LenientPostmortemDraftRule>,
            Rc::new(LenientPostmortemDraftScopeLengthRule { max_length: 32 }),
            Rc::new(LenientPostmortemDraftWindowDaysRangeRule { min: 120, max: 1000 }),
        ],
    }
}
