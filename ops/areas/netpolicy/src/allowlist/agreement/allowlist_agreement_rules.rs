use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist agreement must pass. Check returns its problems, or none.
pub trait AllowlistAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAgreementExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistAgreementRule for AllowlistAgreementExpiresAtRangeRule {
    fn check(&self, item: &AllowlistAgreement) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAgreementCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistAgreementRule for AllowlistAgreementCodeLengthRule {
    fn check(&self, item: &AllowlistAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAgreementAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AllowlistAgreementRule for AllowlistAgreementAllowedNameRule {
    fn check(&self, item: &AllowlistAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAgreementRequiredCodeRule {
    pub message: String,
}

impl AllowlistAgreementRule for AllowlistAgreementRequiredCodeRule {
    fn check(&self, item: &AllowlistAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist agreement.
#[derive(Debug, Clone)]
pub struct AllowlistAgreementValidator {
    pub rules: Vec<Rc<dyn AllowlistAgreementRule>>,
}

impl AllowlistAgreementValidator {
    pub fn validate(&self, item: &AllowlistAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_agreement_validator() -> AllowlistAgreementValidator {
    AllowlistAgreementValidator {
        rules: vec![
            Rc::new(AllowlistAgreementExpiresAtRangeRule { min: 80, max: 3600 }) as Rc<dyn AllowlistAgreementRule>,
            Rc::new(AllowlistAgreementCodeLengthRule { max_length: 64 }),
            Rc::new(AllowlistAgreementAllowedNameRule {
                allowed: vec!["capped".to_string(), "manual".to_string(), "global".to_string()],
            }),
            Rc::new(AllowlistAgreementRequiredCodeRule {
                message: "allowlist agreement needs an entry".to_string(),
            }),
        ],
    }
}
