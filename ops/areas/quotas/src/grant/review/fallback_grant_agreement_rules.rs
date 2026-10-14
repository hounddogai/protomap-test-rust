use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback grant agreement must pass. Check returns its problems, or none.
pub trait FallbackGrantAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FallbackGrantAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackGrantAgreementBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackGrantAgreementRule for FallbackGrantAgreementBurstRangeRule {
    fn check(&self, item: &FallbackGrantAgreement) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackGrantAgreementLabelLengthRule {
    pub max_length: i64,
}

impl FallbackGrantAgreementRule for FallbackGrantAgreementLabelLengthRule {
    fn check(&self, item: &FallbackGrantAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackGrantAgreementAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FallbackGrantAgreementRule for FallbackGrantAgreementAllowedLabelRule {
    fn check(&self, item: &FallbackGrantAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is expired", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback grant agreement.
#[derive(Debug, Clone)]
pub struct FallbackGrantAgreementValidator {
    pub rules: Vec<Rc<dyn FallbackGrantAgreementRule>>,
}

impl FallbackGrantAgreementValidator {
    pub fn validate(&self, item: &FallbackGrantAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackGrantAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackGrantAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_grant_agreement_validator() -> FallbackGrantAgreementValidator {
    FallbackGrantAgreementValidator {
        rules: vec![
            Rc::new(FallbackGrantAgreementBurstRangeRule { min: 80, max: 255 }) as Rc<dyn FallbackGrantAgreementRule>,
            Rc::new(FallbackGrantAgreementLabelLengthRule { max_length: 80 }),
            Rc::new(FallbackGrantAgreementAllowedLabelRule {
                allowed: vec!["regional".to_string(), "strict".to_string(), "default".to_string()],
            }),
        ],
    }
}
