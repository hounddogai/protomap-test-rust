use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual grant agreement must pass. Check returns its problems, or none.
pub trait ManualGrantAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreementCodeLengthRule {
    pub max_length: i64,
}

impl ManualGrantAgreementRule for ManualGrantAgreementCodeLengthRule {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreementCodePrefixRule {
    pub prefix: String,
}

impl ManualGrantAgreementRule for ManualGrantAgreementCodePrefixRule {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreementAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl ManualGrantAgreementRule for ManualGrantAgreementAllowedNotesRule {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is too long", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreementNotesLengthRule {
    pub max_length: i64,
}

impl ManualGrantAgreementRule for ManualGrantAgreementNotesLengthRule {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreementNameLengthRule {
    pub max_length: i64,
}

impl ManualGrantAgreementRule for ManualGrantAgreementNameLengthRule {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantAgreementMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualGrantAgreementRule for ManualGrantAgreementMinimumRangeRule {
    fn check(&self, item: &ManualGrantAgreement) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual grant agreement.
#[derive(Debug, Clone)]
pub struct ManualGrantAgreementValidator {
    pub rules: Vec<Rc<dyn ManualGrantAgreementRule>>,
}

impl ManualGrantAgreementValidator {
    pub fn validate(&self, item: &ManualGrantAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualGrantAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualGrantAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_grant_agreement_validator() -> ManualGrantAgreementValidator {
    ManualGrantAgreementValidator {
        rules: vec![
            Rc::new(ManualGrantAgreementCodeLengthRule { max_length: 16 }) as Rc<dyn ManualGrantAgreementRule>,
            Rc::new(ManualGrantAgreementCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(ManualGrantAgreementAllowedNotesRule {
                allowed: vec!["weighted".to_string(), "manual".to_string(), "default".to_string()],
            }),
            Rc::new(ManualGrantAgreementNotesLengthRule { max_length: 86400 }),
            Rc::new(ManualGrantAgreementNameLengthRule { max_length: 80 }),
            Rc::new(ManualGrantAgreementMinimumRangeRule { min: 7.1, max: 2266.8 }),
        ],
    }
}
