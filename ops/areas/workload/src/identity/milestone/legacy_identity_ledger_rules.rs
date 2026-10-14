use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy identity ledger must pass. Check returns its problems, or none.
pub trait LegacyIdentityLedgerRule: std::fmt::Debug {
    fn check(&self, item: &LegacyIdentityLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerRequiredExternalRefRule {
    pub message: String,
}

impl LegacyIdentityLedgerRule for LegacyIdentityLedgerRequiredExternalRefRule {
    fn check(&self, item: &LegacyIdentityLedger) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyIdentityLedgerRule for LegacyIdentityLedgerMaximumRangeRule {
    fn check(&self, item: &LegacyIdentityLedger) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerChannelLengthRule {
    pub max_length: i64,
}

impl LegacyIdentityLedgerRule for LegacyIdentityLedgerChannelLengthRule {
    fn check(&self, item: &LegacyIdentityLedger) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerRequiredChannelRule {
    pub message: String,
}

impl LegacyIdentityLedgerRule for LegacyIdentityLedgerRequiredChannelRule {
    fn check(&self, item: &LegacyIdentityLedger) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityLedgerCodePrefixRule {
    pub prefix: String,
}

impl LegacyIdentityLedgerRule for LegacyIdentityLedgerCodePrefixRule {
    fn check(&self, item: &LegacyIdentityLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy identity ledger.
#[derive(Debug, Clone)]
pub struct LegacyIdentityLedgerValidator {
    pub rules: Vec<Rc<dyn LegacyIdentityLedgerRule>>,
}

impl LegacyIdentityLedgerValidator {
    pub fn validate(&self, item: &LegacyIdentityLedger) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyIdentityLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyIdentityLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_identity_ledger_validator() -> LegacyIdentityLedgerValidator {
    LegacyIdentityLedgerValidator {
        rules: vec![
            Rc::new(LegacyIdentityLedgerRequiredExternalRefRule {
                message: "legacy identity ledger needs this field".to_string(),
            }) as Rc<dyn LegacyIdentityLedgerRule>,
            Rc::new(LegacyIdentityLedgerMaximumRangeRule { min: 10.0, max: 1390.1 }),
            Rc::new(LegacyIdentityLedgerChannelLengthRule { max_length: 64 }),
            Rc::new(LegacyIdentityLedgerRequiredChannelRule {
                message: "legacy identity ledger needs a value".to_string(),
            }),
            Rc::new(LegacyIdentityLedgerCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
