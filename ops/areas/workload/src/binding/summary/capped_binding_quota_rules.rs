use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped binding quota must pass. Check returns its problems, or none.
pub trait CappedBindingQuotaRule: std::fmt::Debug {
    fn check(&self, item: &CappedBindingQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBindingQuotaVisibleGuardRule {
    pub limit: f64,
}

impl CappedBindingQuotaRule for CappedBindingQuotaVisibleGuardRule {
    fn check(&self, item: &CappedBindingQuota) -> Vec<String> {
        if item.visible && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while visible", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBindingQuotaRequiredSummaryRule {
    pub message: String,
}

impl CappedBindingQuotaRule for CappedBindingQuotaRequiredSummaryRule {
    fn check(&self, item: &CappedBindingQuota) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBindingQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedBindingQuotaRule for CappedBindingQuotaAllowedCodeRule {
    fn check(&self, item: &CappedBindingQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedBindingQuotaFingerprintLengthRule {
    pub max_length: i64,
}

impl CappedBindingQuotaRule for CappedBindingQuotaFingerprintLengthRule {
    fn check(&self, item: &CappedBindingQuota) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped binding quota.
#[derive(Debug, Clone)]
pub struct CappedBindingQuotaValidator {
    pub rules: Vec<Rc<dyn CappedBindingQuotaRule>>,
}

impl CappedBindingQuotaValidator {
    pub fn validate(&self, item: &CappedBindingQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedBindingQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedBindingQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_binding_quota_validator() -> CappedBindingQuotaValidator {
    CappedBindingQuotaValidator {
        rules: vec![
            Rc::new(CappedBindingQuotaVisibleGuardRule { limit: 9.0 }) as Rc<dyn CappedBindingQuotaRule>,
            Rc::new(CappedBindingQuotaRequiredSummaryRule {
                message: "capped binding quota needs this field".to_string(),
            }),
            Rc::new(CappedBindingQuotaAllowedCodeRule {
                allowed: vec!["global".to_string(), "legacy".to_string(), "fallback".to_string()],
            }),
            Rc::new(CappedBindingQuotaFingerprintLengthRule { max_length: 32 }),
        ],
    }
}
