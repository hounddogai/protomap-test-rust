use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped query agreement must pass. Check returns its problems, or none.
pub trait CappedQueryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &CappedQueryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAgreementRequiredExternalRefRule {
    pub message: String,
}

impl CappedQueryAgreementRule for CappedQueryAgreementRequiredExternalRefRule {
    fn check(&self, item: &CappedQueryAgreement) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAgreementExternalRefLengthRule {
    pub max_length: i64,
}

impl CappedQueryAgreementRule for CappedQueryAgreementExternalRefLengthRule {
    fn check(&self, item: &CappedQueryAgreement) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAgreementCodePrefixRule {
    pub prefix: String,
}

impl CappedQueryAgreementRule for CappedQueryAgreementCodePrefixRule {
    fn check(&self, item: &CappedQueryAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped query agreement.
#[derive(Debug, Clone)]
pub struct CappedQueryAgreementValidator {
    pub rules: Vec<Rc<dyn CappedQueryAgreementRule>>,
}

impl CappedQueryAgreementValidator {
    pub fn validate(&self, item: &CappedQueryAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedQueryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQueryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_query_agreement_validator() -> CappedQueryAgreementValidator {
    CappedQueryAgreementValidator {
        rules: vec![
            Rc::new(CappedQueryAgreementRequiredExternalRefRule {
                message: "capped query agreement needs a value".to_string(),
            }) as Rc<dyn CappedQueryAgreementRule>,
            Rc::new(CappedQueryAgreementExternalRefLengthRule { max_length: 1000 }),
            Rc::new(CappedQueryAgreementCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}
