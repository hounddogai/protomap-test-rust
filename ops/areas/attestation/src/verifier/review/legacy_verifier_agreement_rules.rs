use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy verifier agreement must pass. Check returns its problems, or none.
pub trait LegacyVerifierAgreementRule: std::fmt::Debug {
    fn check(&self, item: &LegacyVerifierAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierAgreementCodeLengthRule {
    pub max_length: i64,
}

impl LegacyVerifierAgreementRule for LegacyVerifierAgreementCodeLengthRule {
    fn check(&self, item: &LegacyVerifierAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierAgreementMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyVerifierAgreementRule for LegacyVerifierAgreementMinimumRangeRule {
    fn check(&self, item: &LegacyVerifierAgreement) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVerifierAgreementCodePrefixRule {
    pub prefix: String,
}

impl LegacyVerifierAgreementRule for LegacyVerifierAgreementCodePrefixRule {
    fn check(&self, item: &LegacyVerifierAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy verifier agreement.
#[derive(Debug, Clone)]
pub struct LegacyVerifierAgreementValidator {
    pub rules: Vec<Rc<dyn LegacyVerifierAgreementRule>>,
}

impl LegacyVerifierAgreementValidator {
    pub fn validate(&self, item: &LegacyVerifierAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyVerifierAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyVerifierAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_verifier_agreement_validator() -> LegacyVerifierAgreementValidator {
    LegacyVerifierAgreementValidator {
        rules: vec![
            Rc::new(LegacyVerifierAgreementCodeLengthRule { max_length: 255 }) as Rc<dyn LegacyVerifierAgreementRule>,
            Rc::new(LegacyVerifierAgreementMinimumRangeRule { min: 4.9, max: 2133.0 }),
            Rc::new(LegacyVerifierAgreementCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
