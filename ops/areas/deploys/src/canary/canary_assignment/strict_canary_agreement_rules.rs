use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict canary agreement must pass. Check returns its problems, or none.
pub trait StrictCanaryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementCodePrefixRule {
    pub prefix: String,
}

impl StrictCanaryAgreementRule for StrictCanaryAgreementCodePrefixRule {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementAllowedNameRule {
    pub allowed: Vec<String>,
}

impl StrictCanaryAgreementRule for StrictCanaryAgreementAllowedNameRule {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String> {
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
pub struct StrictCanaryAgreementRequiredSummaryRule {
    pub message: String,
}

impl StrictCanaryAgreementRule for StrictCanaryAgreementRequiredSummaryRule {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementRequiredKeyIdRule {
    pub message: String,
}

impl StrictCanaryAgreementRule for StrictCanaryAgreementRequiredKeyIdRule {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementRequiredNameRule {
    pub message: String,
}

impl StrictCanaryAgreementRule for StrictCanaryAgreementRequiredNameRule {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictCanaryAgreementRule for StrictCanaryAgreementUpdatedAtRangeRule {
    fn check(&self, item: &StrictCanaryAgreement) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict canary agreement.
#[derive(Debug, Clone)]
pub struct StrictCanaryAgreementValidator {
    pub rules: Vec<Rc<dyn StrictCanaryAgreementRule>>,
}

impl StrictCanaryAgreementValidator {
    pub fn validate(&self, item: &StrictCanaryAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictCanaryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictCanaryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_canary_agreement_validator() -> StrictCanaryAgreementValidator {
    StrictCanaryAgreementValidator {
        rules: vec![
            Rc::new(StrictCanaryAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn StrictCanaryAgreementRule>,
            Rc::new(StrictCanaryAgreementAllowedNameRule { allowed: Vec::new() }),
            Rc::new(StrictCanaryAgreementRequiredSummaryRule {
                message: "strict canary agreement needs a value".to_string(),
            }),
            Rc::new(StrictCanaryAgreementRequiredKeyIdRule {
                message: "strict canary agreement needs an entry".to_string(),
            }),
            Rc::new(StrictCanaryAgreementRequiredNameRule {
                message: "strict canary agreement needs this field".to_string(),
            }),
            Rc::new(StrictCanaryAgreementUpdatedAtRangeRule { min: 255, max: 255 }),
        ],
    }
}
