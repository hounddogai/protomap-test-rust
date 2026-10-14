use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive credential variance must pass. Check returns its problems, or none.
pub trait AdaptiveCredentialVarianceRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveCredentialVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVariancePinnedGuardRule {
    pub limit: i64,
}

impl AdaptiveCredentialVarianceRule for AdaptiveCredentialVariancePinnedGuardRule {
    fn check(&self, item: &AdaptiveCredentialVariance) -> Vec<String> {
        if item.pinned && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while pinned",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVarianceCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveCredentialVarianceRule for AdaptiveCredentialVarianceCodePrefixRule {
    fn check(&self, item: &AdaptiveCredentialVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVarianceRequiredCodeRule {
    pub message: String,
}

impl AdaptiveCredentialVarianceRule for AdaptiveCredentialVarianceRequiredCodeRule {
    fn check(&self, item: &AdaptiveCredentialVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCredentialVarianceVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveCredentialVarianceRule for AdaptiveCredentialVarianceVersionRangeRule {
    fn check(&self, item: &AdaptiveCredentialVariance) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive credential variance.
#[derive(Debug, Clone)]
pub struct AdaptiveCredentialVarianceValidator {
    pub rules: Vec<Rc<dyn AdaptiveCredentialVarianceRule>>,
}

impl AdaptiveCredentialVarianceValidator {
    pub fn validate(&self, item: &AdaptiveCredentialVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveCredentialVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveCredentialVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_credential_variance_validator() -> AdaptiveCredentialVarianceValidator {
    AdaptiveCredentialVarianceValidator {
        rules: vec![
            Rc::new(AdaptiveCredentialVariancePinnedGuardRule { limit: 32 }) as Rc<dyn AdaptiveCredentialVarianceRule>,
            Rc::new(AdaptiveCredentialVarianceCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AdaptiveCredentialVarianceRequiredCodeRule {
                message: "adaptive credential variance needs this field".to_string(),
            }),
            Rc::new(AdaptiveCredentialVarianceVersionRangeRule { min: 3600, max: 86400 }),
        ],
    }
}
