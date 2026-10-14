use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback plan digest must pass. Check returns its problems, or none.
pub trait FallbackPlanDigestRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPlanDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigestAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl FallbackPlanDigestRule for FallbackPlanDigestAllowedCategoryRule {
    fn check(&self, item: &FallbackPlanDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is too long", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigestCurrencyLengthRule {
    pub max_length: i64,
}

impl FallbackPlanDigestRule for FallbackPlanDigestCurrencyLengthRule {
    fn check(&self, item: &FallbackPlanDigest) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigestCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPlanDigestRule for FallbackPlanDigestCreatedAtRangeRule {
    fn check(&self, item: &FallbackPlanDigest) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanDigestNameLengthRule {
    pub max_length: i64,
}

impl FallbackPlanDigestRule for FallbackPlanDigestNameLengthRule {
    fn check(&self, item: &FallbackPlanDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback plan digest.
#[derive(Debug, Clone)]
pub struct FallbackPlanDigestValidator {
    pub rules: Vec<Rc<dyn FallbackPlanDigestRule>>,
}

impl FallbackPlanDigestValidator {
    pub fn validate(&self, item: &FallbackPlanDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPlanDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPlanDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_plan_digest_validator() -> FallbackPlanDigestValidator {
    FallbackPlanDigestValidator {
        rules: vec![
            Rc::new(FallbackPlanDigestAllowedCategoryRule {
                allowed: vec!["batched".to_string(), "scheduled".to_string()],
            }) as Rc<dyn FallbackPlanDigestRule>,
            Rc::new(FallbackPlanDigestCurrencyLengthRule { max_length: 120 }),
            Rc::new(FallbackPlanDigestCreatedAtRangeRule { min: 120, max: 3600 }),
            Rc::new(FallbackPlanDigestNameLengthRule { max_length: 32 }),
        ],
    }
}
