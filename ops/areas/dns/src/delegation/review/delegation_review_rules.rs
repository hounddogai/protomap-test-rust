use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation review must pass. Check returns its problems, or none.
pub trait DelegationReviewRule: std::fmt::Debug {
    fn check(&self, item: &DelegationReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReviewCodePrefixRule {
    pub prefix: String,
}

impl DelegationReviewRule for DelegationReviewCodePrefixRule {
    fn check(&self, item: &DelegationReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReviewRequiredKeyIdRule {
    pub message: String,
}

impl DelegationReviewRule for DelegationReviewRequiredKeyIdRule {
    fn check(&self, item: &DelegationReview) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReviewDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationReviewRule for DelegationReviewDiscountRangeRule {
    fn check(&self, item: &DelegationReview) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReviewCodeLengthRule {
    pub max_length: i64,
}

impl DelegationReviewRule for DelegationReviewCodeLengthRule {
    fn check(&self, item: &DelegationReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationReviewRequiredPhoneRule {
    pub message: String,
}

impl DelegationReviewRule for DelegationReviewRequiredPhoneRule {
    fn check(&self, item: &DelegationReview) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation review.
#[derive(Debug, Clone)]
pub struct DelegationReviewValidator {
    pub rules: Vec<Rc<dyn DelegationReviewRule>>,
}

impl DelegationReviewValidator {
    pub fn validate(&self, item: &DelegationReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_review_validator() -> DelegationReviewValidator {
    DelegationReviewValidator {
        rules: vec![
            Rc::new(DelegationReviewCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn DelegationReviewRule>,
            Rc::new(DelegationReviewRequiredKeyIdRule {
                message: "delegation review needs this field".to_string(),
            }),
            Rc::new(DelegationReviewDiscountRangeRule { min: 5.0, max: 1599.0 }),
            Rc::new(DelegationReviewCodeLengthRule { max_length: 3600 }),
            Rc::new(DelegationReviewRequiredPhoneRule {
                message: "delegation review needs this field".to_string(),
            }),
        ],
    }
}
