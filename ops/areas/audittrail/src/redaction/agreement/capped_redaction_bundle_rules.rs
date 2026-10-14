use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped redaction bundle must pass. Check returns its problems, or none.
pub trait CappedRedactionBundleRule: std::fmt::Debug {
    fn check(&self, item: &CappedRedactionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionBundleGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedRedactionBundleRule for CappedRedactionBundleGraceDaysRangeRule {
    fn check(&self, item: &CappedRedactionBundle) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionBundleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl CappedRedactionBundleRule for CappedRedactionBundleAllowedLabelRule {
    fn check(&self, item: &CappedRedactionBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRedactionBundleRequiredLabelRule {
    pub message: String,
}

impl CappedRedactionBundleRule for CappedRedactionBundleRequiredLabelRule {
    fn check(&self, item: &CappedRedactionBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped redaction bundle.
#[derive(Debug, Clone)]
pub struct CappedRedactionBundleValidator {
    pub rules: Vec<Rc<dyn CappedRedactionBundleRule>>,
}

impl CappedRedactionBundleValidator {
    pub fn validate(&self, item: &CappedRedactionBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedRedactionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedRedactionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_redaction_bundle_validator() -> CappedRedactionBundleValidator {
    CappedRedactionBundleValidator {
        rules: vec![
            Rc::new(CappedRedactionBundleGraceDaysRangeRule { min: 3600, max: 3600 })
                as Rc<dyn CappedRedactionBundleRule>,
            Rc::new(CappedRedactionBundleAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(CappedRedactionBundleRequiredLabelRule {
                message: "capped redaction bundle needs an entry".to_string(),
            }),
        ],
    }
}
