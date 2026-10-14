use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy trust domain estimate must pass. Check returns its problems, or none.
pub trait LegacyTrustDomainEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyTrustDomainEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainEstimateCodePrefixRule {
    pub prefix: String,
}

impl LegacyTrustDomainEstimateRule for LegacyTrustDomainEstimateCodePrefixRule {
    fn check(&self, item: &LegacyTrustDomainEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainEstimateAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl LegacyTrustDomainEstimateRule for LegacyTrustDomainEstimateAllowedUnitRule {
    fn check(&self, item: &LegacyTrustDomainEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is above maximum", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainEstimateUnitLengthRule {
    pub max_length: i64,
}

impl LegacyTrustDomainEstimateRule for LegacyTrustDomainEstimateUnitLengthRule {
    fn check(&self, item: &LegacyTrustDomainEstimate) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainEstimateVisibleGuardRule {
    pub limit: i64,
}

impl LegacyTrustDomainEstimateRule for LegacyTrustDomainEstimateVisibleGuardRule {
    fn check(&self, item: &LegacyTrustDomainEstimate) -> Vec<String> {
        if item.visible && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while visible", item.version, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy trust domain estimate.
#[derive(Debug, Clone)]
pub struct LegacyTrustDomainEstimateValidator {
    pub rules: Vec<Rc<dyn LegacyTrustDomainEstimateRule>>,
}

impl LegacyTrustDomainEstimateValidator {
    pub fn validate(&self, item: &LegacyTrustDomainEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyTrustDomainEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyTrustDomainEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_trust_domain_estimate_validator() -> LegacyTrustDomainEstimateValidator {
    LegacyTrustDomainEstimateValidator {
        rules: vec![
            Rc::new(LegacyTrustDomainEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn LegacyTrustDomainEstimateRule>,
            Rc::new(LegacyTrustDomainEstimateAllowedUnitRule {
                allowed: vec!["seasonal".to_string(), "weighted".to_string()],
            }),
            Rc::new(LegacyTrustDomainEstimateUnitLengthRule { max_length: 1000 }),
            Rc::new(LegacyTrustDomainEstimateVisibleGuardRule { limit: 86400 }),
        ],
    }
}
