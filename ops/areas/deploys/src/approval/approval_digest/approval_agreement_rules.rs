use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval agreement must pass. Check returns its problems, or none.
pub trait ApprovalAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl ApprovalAgreementRule for ApprovalAgreementAllowedContactEmailRule {
    fn check(&self, item: &ApprovalAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is stale", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementRequiredCountryRule {
    pub message: String,
}

impl ApprovalAgreementRule for ApprovalAgreementRequiredCountryRule {
    fn check(&self, item: &ApprovalAgreement) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementCountryLengthRule {
    pub max_length: i64,
}

impl ApprovalAgreementRule for ApprovalAgreementCountryLengthRule {
    fn check(&self, item: &ApprovalAgreement) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalAgreementRule for ApprovalAgreementScoreRangeRule {
    fn check(&self, item: &ApprovalAgreement) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a approval agreement.
#[derive(Debug, Clone)]
pub struct ApprovalAgreementValidator {
    pub rules: Vec<Rc<dyn ApprovalAgreementRule>>,
}

impl ApprovalAgreementValidator {
    pub fn validate(&self, item: &ApprovalAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ApprovalAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_agreement_validator() -> ApprovalAgreementValidator {
    ApprovalAgreementValidator {
        rules: vec![
            Rc::new(ApprovalAgreementAllowedContactEmailRule {
                allowed: vec!["seasonal".to_string(), "seasonal".to_string(), "weighted".to_string()],
            }) as Rc<dyn ApprovalAgreementRule>,
            Rc::new(ApprovalAgreementRequiredCountryRule {
                message: "approval agreement needs an entry".to_string(),
            }),
            Rc::new(ApprovalAgreementCountryLengthRule { max_length: 32 }),
            Rc::new(ApprovalAgreementScoreRangeRule { min: 7.5, max: 1170.7 }),
        ],
    }
}
