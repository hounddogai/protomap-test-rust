use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched federation ledger must pass. Check returns its problems, or none.
pub trait BatchedFederationLedgerRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFederationLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedgerMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFederationLedgerRule for BatchedFederationLedgerMaximumRangeRule {
    fn check(&self, item: &BatchedFederationLedger) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedgerRequiredChannelRule {
    pub message: String,
}

impl BatchedFederationLedgerRule for BatchedFederationLedgerRequiredChannelRule {
    fn check(&self, item: &BatchedFederationLedger) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedgerChannelLengthRule {
    pub max_length: i64,
}

impl BatchedFederationLedgerRule for BatchedFederationLedgerChannelLengthRule {
    fn check(&self, item: &BatchedFederationLedger) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedgerRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFederationLedgerRule for BatchedFederationLedgerRateRangeRule {
    fn check(&self, item: &BatchedFederationLedger) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationLedgerCodePrefixRule {
    pub prefix: String,
}

impl BatchedFederationLedgerRule for BatchedFederationLedgerCodePrefixRule {
    fn check(&self, item: &BatchedFederationLedger) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched federation ledger.
#[derive(Debug, Clone)]
pub struct BatchedFederationLedgerValidator {
    pub rules: Vec<Rc<dyn BatchedFederationLedgerRule>>,
}

impl BatchedFederationLedgerValidator {
    pub fn validate(&self, item: &BatchedFederationLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedFederationLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFederationLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_federation_ledger_validator() -> BatchedFederationLedgerValidator {
    BatchedFederationLedgerValidator {
        rules: vec![
            Rc::new(BatchedFederationLedgerMaximumRangeRule { min: 0.2, max: 3441.9 })
                as Rc<dyn BatchedFederationLedgerRule>,
            Rc::new(BatchedFederationLedgerRequiredChannelRule {
                message: "batched federation ledger needs an entry".to_string(),
            }),
            Rc::new(BatchedFederationLedgerChannelLengthRule { max_length: 120 }),
            Rc::new(BatchedFederationLedgerRateRangeRule { min: 3.6, max: 4303.2 }),
            Rc::new(BatchedFederationLedgerCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}
