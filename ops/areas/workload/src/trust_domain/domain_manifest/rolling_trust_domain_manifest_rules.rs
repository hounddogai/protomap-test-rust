use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling trust domain manifest must pass. Check returns its problems, or none.
pub trait RollingTrustDomainManifestRule: std::fmt::Debug {
    fn check(&self, item: &RollingTrustDomainManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTrustDomainManifestRequiredCodeRule {
    pub message: String,
}

impl RollingTrustDomainManifestRule for RollingTrustDomainManifestRequiredCodeRule {
    fn check(&self, item: &RollingTrustDomainManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTrustDomainManifestAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollingTrustDomainManifestRule for RollingTrustDomainManifestAllowedNameRule {
    fn check(&self, item: &RollingTrustDomainManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTrustDomainManifestRequiredTitleRule {
    pub message: String,
}

impl RollingTrustDomainManifestRule for RollingTrustDomainManifestRequiredTitleRule {
    fn check(&self, item: &RollingTrustDomainManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTrustDomainManifestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingTrustDomainManifestRule for RollingTrustDomainManifestMaximumRangeRule {
    fn check(&self, item: &RollingTrustDomainManifest) -> Vec<String> {
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

/// Runs every rule of a rolling trust domain manifest.
#[derive(Debug, Clone)]
pub struct RollingTrustDomainManifestValidator {
    pub rules: Vec<Rc<dyn RollingTrustDomainManifestRule>>,
}

impl RollingTrustDomainManifestValidator {
    pub fn validate(&self, item: &RollingTrustDomainManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingTrustDomainManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTrustDomainManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_trust_domain_manifest_validator() -> RollingTrustDomainManifestValidator {
    RollingTrustDomainManifestValidator {
        rules: vec![
            Rc::new(RollingTrustDomainManifestRequiredCodeRule {
                message: "rolling trust domain manifest needs this field".to_string(),
            }) as Rc<dyn RollingTrustDomainManifestRule>,
            Rc::new(RollingTrustDomainManifestAllowedNameRule {
                allowed: vec!["pending".to_string(), "lenient".to_string(), "legacy".to_string()],
            }),
            Rc::new(RollingTrustDomainManifestRequiredTitleRule {
                message: "rolling trust domain manifest needs a value".to_string(),
            }),
            Rc::new(RollingTrustDomainManifestMaximumRangeRule { min: 1.3, max: 1837.6 }),
        ],
    }
}
