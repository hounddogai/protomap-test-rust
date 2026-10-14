use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary nonce baseline must pass. Check returns its problems, or none.
pub trait PrimaryNonceBaselineRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryNonceBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceBaselineCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryNonceBaselineRule for PrimaryNonceBaselineCreatedAtRangeRule {
    fn check(&self, item: &PrimaryNonceBaseline) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceBaselineRequiredPhoneRule {
    pub message: String,
}

impl PrimaryNonceBaselineRule for PrimaryNonceBaselineRequiredPhoneRule {
    fn check(&self, item: &PrimaryNonceBaseline) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceBaselineAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl PrimaryNonceBaselineRule for PrimaryNonceBaselineAllowedPhoneRule {
    fn check(&self, item: &PrimaryNonceBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is duplicated", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceBaselineArchivedGuardRule {
    pub limit: i64,
}

impl PrimaryNonceBaselineRule for PrimaryNonceBaselineArchivedGuardRule {
    fn check(&self, item: &PrimaryNonceBaseline) -> Vec<String> {
        if item.archived && (item.created_at > self.limit) {
            return vec![format!(
                "created at {} exceeds {} while archived",
                item.created_at, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary nonce baseline.
#[derive(Debug, Clone)]
pub struct PrimaryNonceBaselineValidator {
    pub rules: Vec<Rc<dyn PrimaryNonceBaselineRule>>,
}

impl PrimaryNonceBaselineValidator {
    pub fn validate(&self, item: &PrimaryNonceBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryNonceBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryNonceBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_nonce_baseline_validator() -> PrimaryNonceBaselineValidator {
    PrimaryNonceBaselineValidator {
        rules: vec![
            Rc::new(PrimaryNonceBaselineCreatedAtRangeRule { min: 80, max: 255 }) as Rc<dyn PrimaryNonceBaselineRule>,
            Rc::new(PrimaryNonceBaselineRequiredPhoneRule {
                message: "primary nonce baseline needs a value".to_string(),
            }),
            Rc::new(PrimaryNonceBaselineAllowedPhoneRule {
                allowed: vec!["regional".to_string()],
            }),
            Rc::new(PrimaryNonceBaselineArchivedGuardRule { limit: 86400 }),
        ],
    }
}
