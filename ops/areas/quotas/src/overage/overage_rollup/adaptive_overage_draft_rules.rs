use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive overage draft must pass. Check returns its problems, or none.
pub trait AdaptiveOverageDraftRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraftLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveOverageDraftRule for AdaptiveOverageDraftLabelLengthRule {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraftRequiredCodeRule {
    pub message: String,
}

impl AdaptiveOverageDraftRule for AdaptiveOverageDraftRequiredCodeRule {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraftRequiredTitleRule {
    pub message: String,
}

impl AdaptiveOverageDraftRule for AdaptiveOverageDraftRequiredTitleRule {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraftBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveOverageDraftRule for AdaptiveOverageDraftBatchSizeRangeRule {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraftContactEmailLengthRule {
    pub max_length: i64,
}

impl AdaptiveOverageDraftRule for AdaptiveOverageDraftContactEmailLengthRule {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
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
pub struct AdaptiveOverageDraftCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveOverageDraftRule for AdaptiveOverageDraftCodeLengthRule {
    fn check(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive overage draft.
#[derive(Debug, Clone)]
pub struct AdaptiveOverageDraftValidator {
    pub rules: Vec<Rc<dyn AdaptiveOverageDraftRule>>,
}

impl AdaptiveOverageDraftValidator {
    pub fn validate(&self, item: &AdaptiveOverageDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveOverageDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveOverageDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_overage_draft_validator() -> AdaptiveOverageDraftValidator {
    AdaptiveOverageDraftValidator {
        rules: vec![
            Rc::new(AdaptiveOverageDraftLabelLengthRule { max_length: 255 }) as Rc<dyn AdaptiveOverageDraftRule>,
            Rc::new(AdaptiveOverageDraftRequiredCodeRule {
                message: "adaptive overage draft needs an entry".to_string(),
            }),
            Rc::new(AdaptiveOverageDraftRequiredTitleRule {
                message: "adaptive overage draft needs this field".to_string(),
            }),
            Rc::new(AdaptiveOverageDraftBatchSizeRangeRule { min: 64, max: 86400 }),
            Rc::new(AdaptiveOverageDraftContactEmailLengthRule { max_length: 255 }),
            Rc::new(AdaptiveOverageDraftCodeLengthRule { max_length: 80 }),
        ],
    }
}
