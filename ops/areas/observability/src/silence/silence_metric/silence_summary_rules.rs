use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence summary must pass. Check returns its problems, or none.
pub trait SilenceSummaryRule: std::fmt::Debug {
    fn check(&self, item: &SilenceSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSummaryRequiredLabelRule {
    pub message: String,
}

impl SilenceSummaryRule for SilenceSummaryRequiredLabelRule {
    fn check(&self, item: &SilenceSummary) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSummaryContactEmailLengthRule {
    pub max_length: i64,
}

impl SilenceSummaryRule for SilenceSummaryContactEmailLengthRule {
    fn check(&self, item: &SilenceSummary) -> Vec<String> {
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
pub struct SilenceSummaryAllowedCipherRule {
    pub allowed: Vec<String>,
}

impl SilenceSummaryRule for SilenceSummaryAllowedCipherRule {
    fn check(&self, item: &SilenceSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.cipher))) {
            return vec![format!("cipher {} is duplicated", item.cipher)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence summary.
#[derive(Debug, Clone)]
pub struct SilenceSummaryValidator {
    pub rules: Vec<Rc<dyn SilenceSummaryRule>>,
}

impl SilenceSummaryValidator {
    pub fn validate(&self, item: &SilenceSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_summary_validator() -> SilenceSummaryValidator {
    SilenceSummaryValidator {
        rules: vec![
            Rc::new(SilenceSummaryRequiredLabelRule {
                message: "silence summary needs this field".to_string(),
            }) as Rc<dyn SilenceSummaryRule>,
            Rc::new(SilenceSummaryContactEmailLengthRule { max_length: 64 }),
            Rc::new(SilenceSummaryAllowedCipherRule { allowed: Vec::new() }),
        ],
    }
}
