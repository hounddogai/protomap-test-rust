use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict sink agreement must pass. Check returns its problems, or none.
pub trait StrictSinkAgreementRule: std::fmt::Debug {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementRequiredTenantIdRule {
    pub message: String,
}

impl StrictSinkAgreementRule for StrictSinkAgreementRequiredTenantIdRule {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementTenantIdLengthRule {
    pub max_length: i64,
}

impl StrictSinkAgreementRule for StrictSinkAgreementTenantIdLengthRule {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementRequiredTitleRule {
    pub message: String,
}

impl StrictSinkAgreementRule for StrictSinkAgreementRequiredTitleRule {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementCodePrefixRule {
    pub prefix: String,
}

impl StrictSinkAgreementRule for StrictSinkAgreementCodePrefixRule {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictSinkAgreementRule for StrictSinkAgreementWindowDaysRangeRule {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl StrictSinkAgreementRule for StrictSinkAgreementAllowedTenantIdRule {
    fn check(&self, item: &StrictSinkAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is missing", item.tenant_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict sink agreement.
#[derive(Debug, Clone)]
pub struct StrictSinkAgreementValidator {
    pub rules: Vec<Rc<dyn StrictSinkAgreementRule>>,
}

impl StrictSinkAgreementValidator {
    pub fn validate(&self, item: &StrictSinkAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictSinkAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSinkAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_sink_agreement_validator() -> StrictSinkAgreementValidator {
    StrictSinkAgreementValidator {
        rules: vec![
            Rc::new(StrictSinkAgreementRequiredTenantIdRule {
                message: "strict sink agreement needs this field".to_string(),
            }) as Rc<dyn StrictSinkAgreementRule>,
            Rc::new(StrictSinkAgreementTenantIdLengthRule { max_length: 120 }),
            Rc::new(StrictSinkAgreementRequiredTitleRule {
                message: "strict sink agreement needs an entry".to_string(),
            }),
            Rc::new(StrictSinkAgreementCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictSinkAgreementWindowDaysRangeRule { min: 32, max: 86400 }),
            Rc::new(StrictSinkAgreementAllowedTenantIdRule {
                allowed: vec!["fallback".to_string(), "weighted".to_string(), "rolling".to_string()],
            }),
        ],
    }
}
