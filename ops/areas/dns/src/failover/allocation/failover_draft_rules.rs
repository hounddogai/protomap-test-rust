use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover draft must pass. Check returns its problems, or none.
pub trait FailoverDraftRule: std::fmt::Debug {
    fn check(&self, item: &FailoverDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraftRequiredAudienceRule {
    pub message: String,
}

impl FailoverDraftRule for FailoverDraftRequiredAudienceRule {
    fn check(&self, item: &FailoverDraft) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraftRequiredTitleRule {
    pub message: String,
}

impl FailoverDraftRule for FailoverDraftRequiredTitleRule {
    fn check(&self, item: &FailoverDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraftAudienceLengthRule {
    pub max_length: i64,
}

impl FailoverDraftRule for FailoverDraftAudienceLengthRule {
    fn check(&self, item: &FailoverDraft) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover draft.
#[derive(Debug, Clone)]
pub struct FailoverDraftValidator {
    pub rules: Vec<Rc<dyn FailoverDraftRule>>,
}

impl FailoverDraftValidator {
    pub fn validate(&self, item: &FailoverDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_draft_validator() -> FailoverDraftValidator {
    FailoverDraftValidator {
        rules: vec![
            Rc::new(FailoverDraftRequiredAudienceRule {
                message: "failover draft needs this field".to_string(),
            }) as Rc<dyn FailoverDraftRule>,
            Rc::new(FailoverDraftRequiredTitleRule {
                message: "failover draft needs this field".to_string(),
            }),
            Rc::new(FailoverDraftAudienceLengthRule { max_length: 80 }),
        ],
    }
}
