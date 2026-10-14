use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped trust domain exception must pass. Check returns its problems, or none.
pub trait CappedTrustDomainExceptionRule: std::fmt::Debug {
    fn check(&self, item: &CappedTrustDomainException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainExceptionMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedTrustDomainExceptionRule for CappedTrustDomainExceptionMinimumRangeRule {
    fn check(&self, item: &CappedTrustDomainException) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainExceptionLabelLengthRule {
    pub max_length: i64,
}

impl CappedTrustDomainExceptionRule for CappedTrustDomainExceptionLabelLengthRule {
    fn check(&self, item: &CappedTrustDomainException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainExceptionRequiredNamespaceRule {
    pub message: String,
}

impl CappedTrustDomainExceptionRule for CappedTrustDomainExceptionRequiredNamespaceRule {
    fn check(&self, item: &CappedTrustDomainException) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainExceptionNamespaceLengthRule {
    pub max_length: i64,
}

impl CappedTrustDomainExceptionRule for CappedTrustDomainExceptionNamespaceLengthRule {
    fn check(&self, item: &CappedTrustDomainException) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped trust domain exception.
#[derive(Debug, Clone)]
pub struct CappedTrustDomainExceptionValidator {
    pub rules: Vec<Rc<dyn CappedTrustDomainExceptionRule>>,
}

impl CappedTrustDomainExceptionValidator {
    pub fn validate(&self, item: &CappedTrustDomainException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedTrustDomainException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTrustDomainException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_trust_domain_exception_validator() -> CappedTrustDomainExceptionValidator {
    CappedTrustDomainExceptionValidator {
        rules: vec![
            Rc::new(CappedTrustDomainExceptionMinimumRangeRule { min: 4.1, max: 3010.2 })
                as Rc<dyn CappedTrustDomainExceptionRule>,
            Rc::new(CappedTrustDomainExceptionLabelLengthRule { max_length: 3600 }),
            Rc::new(CappedTrustDomainExceptionRequiredNamespaceRule {
                message: "capped trust domain exception needs an entry".to_string(),
            }),
            Rc::new(CappedTrustDomainExceptionNamespaceLengthRule { max_length: 16 }),
        ],
    }
}
