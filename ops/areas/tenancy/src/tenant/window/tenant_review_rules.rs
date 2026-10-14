use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant review must pass. Check returns its problems, or none.
pub trait TenantReviewRule: std::fmt::Debug {
    fn check(&self, item: &TenantReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantReviewAllowedNameRule {
    pub allowed: Vec<String>,
}

impl TenantReviewRule for TenantReviewAllowedNameRule {
    fn check(&self, item: &TenantReview) -> Vec<String> {
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
pub struct TenantReviewNameLengthRule {
    pub max_length: i64,
}

impl TenantReviewRule for TenantReviewNameLengthRule {
    fn check(&self, item: &TenantReview) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantReviewCodePrefixRule {
    pub prefix: String,
}

impl TenantReviewRule for TenantReviewCodePrefixRule {
    fn check(&self, item: &TenantReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant review.
#[derive(Debug, Clone)]
pub struct TenantReviewValidator {
    pub rules: Vec<Rc<dyn TenantReviewRule>>,
}

impl TenantReviewValidator {
    pub fn validate(&self, item: &TenantReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_review_validator() -> TenantReviewValidator {
    TenantReviewValidator {
        rules: vec![
            Rc::new(TenantReviewAllowedNameRule { allowed: Vec::new() }) as Rc<dyn TenantReviewRule>,
            Rc::new(TenantReviewNameLengthRule { max_length: 80 }),
            Rc::new(TenantReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
