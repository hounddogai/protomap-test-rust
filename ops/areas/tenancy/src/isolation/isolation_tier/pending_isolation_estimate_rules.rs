use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending isolation estimate must pass. Check returns its problems, or none.
pub trait PendingIsolationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &PendingIsolationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimateCurrencyLengthRule {
    pub max_length: i64,
}

impl PendingIsolationEstimateRule for PendingIsolationEstimateCurrencyLengthRule {
    fn check(&self, item: &PendingIsolationEstimate) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimateRequiredCodeRule {
    pub message: String,
}

impl PendingIsolationEstimateRule for PendingIsolationEstimateRequiredCodeRule {
    fn check(&self, item: &PendingIsolationEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimateCodeLengthRule {
    pub max_length: i64,
}

impl PendingIsolationEstimateRule for PendingIsolationEstimateCodeLengthRule {
    fn check(&self, item: &PendingIsolationEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationEstimateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingIsolationEstimateRule for PendingIsolationEstimateAllowedCodeRule {
    fn check(&self, item: &PendingIsolationEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending isolation estimate.
#[derive(Debug, Clone)]
pub struct PendingIsolationEstimateValidator {
    pub rules: Vec<Rc<dyn PendingIsolationEstimateRule>>,
}

impl PendingIsolationEstimateValidator {
    pub fn validate(&self, item: &PendingIsolationEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingIsolationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingIsolationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_isolation_estimate_validator() -> PendingIsolationEstimateValidator {
    PendingIsolationEstimateValidator {
        rules: vec![
            Rc::new(PendingIsolationEstimateCurrencyLengthRule { max_length: 86400 })
                as Rc<dyn PendingIsolationEstimateRule>,
            Rc::new(PendingIsolationEstimateRequiredCodeRule {
                message: "pending isolation estimate needs an entry".to_string(),
            }),
            Rc::new(PendingIsolationEstimateCodeLengthRule { max_length: 120 }),
            Rc::new(PendingIsolationEstimateAllowedCodeRule {
                allowed: vec!["regional".to_string(), "global".to_string()],
            }),
        ],
    }
}
