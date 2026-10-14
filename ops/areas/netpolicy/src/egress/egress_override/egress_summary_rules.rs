use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress summary must pass. Check returns its problems, or none.
pub trait EgressSummaryRule: std::fmt::Debug {
    fn check(&self, item: &EgressSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryRequiredLocaleRule {
    pub message: String,
}

impl EgressSummaryRule for EgressSummaryRequiredLocaleRule {
    fn check(&self, item: &EgressSummary) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EgressSummaryRule for EgressSummaryAllowedCodeRule {
    fn check(&self, item: &EgressSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressSummaryRule for EgressSummaryBurstRangeRule {
    fn check(&self, item: &EgressSummary) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryLocaleLengthRule {
    pub max_length: i64,
}

impl EgressSummaryRule for EgressSummaryLocaleLengthRule {
    fn check(&self, item: &EgressSummary) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryPinnedGuardRule {
    pub limit: i64,
}

impl EgressSummaryRule for EgressSummaryPinnedGuardRule {
    fn check(&self, item: &EgressSummary) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSummaryRequiredCodeRule {
    pub message: String,
}

impl EgressSummaryRule for EgressSummaryRequiredCodeRule {
    fn check(&self, item: &EgressSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress summary.
#[derive(Debug, Clone)]
pub struct EgressSummaryValidator {
    pub rules: Vec<Rc<dyn EgressSummaryRule>>,
}

impl EgressSummaryValidator {
    pub fn validate(&self, item: &EgressSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_summary_validator() -> EgressSummaryValidator {
    EgressSummaryValidator {
        rules: vec![
            Rc::new(EgressSummaryRequiredLocaleRule {
                message: "egress summary needs a value".to_string(),
            }) as Rc<dyn EgressSummaryRule>,
            Rc::new(EgressSummaryAllowedCodeRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(EgressSummaryBurstRangeRule { min: 80, max: 120 }),
            Rc::new(EgressSummaryLocaleLengthRule { max_length: 3600 }),
            Rc::new(EgressSummaryPinnedGuardRule { limit: 86400 }),
            Rc::new(EgressSummaryRequiredCodeRule {
                message: "egress summary needs an entry".to_string(),
            }),
        ],
    }
}
