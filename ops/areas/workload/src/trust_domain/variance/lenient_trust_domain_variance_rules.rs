use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient trust domain variance must pass. Check returns its problems, or none.
pub trait LenientTrustDomainVarianceRule: std::fmt::Debug {
    fn check(&self, item: &LenientTrustDomainVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainVarianceRequiredCodeRule {
    pub message: String,
}

impl LenientTrustDomainVarianceRule for LenientTrustDomainVarianceRequiredCodeRule {
    fn check(&self, item: &LenientTrustDomainVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainVarianceScopeLengthRule {
    pub max_length: i64,
}

impl LenientTrustDomainVarianceRule for LenientTrustDomainVarianceScopeLengthRule {
    fn check(&self, item: &LenientTrustDomainVariance) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainVarianceCodePrefixRule {
    pub prefix: String,
}

impl LenientTrustDomainVarianceRule for LenientTrustDomainVarianceCodePrefixRule {
    fn check(&self, item: &LenientTrustDomainVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientTrustDomainVarianceLabelLengthRule {
    pub max_length: i64,
}

impl LenientTrustDomainVarianceRule for LenientTrustDomainVarianceLabelLengthRule {
    fn check(&self, item: &LenientTrustDomainVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient trust domain variance.
#[derive(Debug, Clone)]
pub struct LenientTrustDomainVarianceValidator {
    pub rules: Vec<Rc<dyn LenientTrustDomainVarianceRule>>,
}

impl LenientTrustDomainVarianceValidator {
    pub fn validate(&self, item: &LenientTrustDomainVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientTrustDomainVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientTrustDomainVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_trust_domain_variance_validator() -> LenientTrustDomainVarianceValidator {
    LenientTrustDomainVarianceValidator {
        rules: vec![
            Rc::new(LenientTrustDomainVarianceRequiredCodeRule {
                message: "lenient trust domain variance needs a value".to_string(),
            }) as Rc<dyn LenientTrustDomainVarianceRule>,
            Rc::new(LenientTrustDomainVarianceScopeLengthRule { max_length: 80 }),
            Rc::new(LenientTrustDomainVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LenientTrustDomainVarianceLabelLengthRule { max_length: 16 }),
        ],
    }
}
