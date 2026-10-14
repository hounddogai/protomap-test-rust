use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a token variance must pass. Check returns its problems, or none.
pub trait TokenVarianceRule: std::fmt::Debug {
    fn check(&self, item: &TokenVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenVarianceAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl TokenVarianceRule for TokenVarianceAllowedExternalRefRule {
    fn check(&self, item: &TokenVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is expired", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenVarianceAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TokenVarianceRule for TokenVarianceAmountRangeRule {
    fn check(&self, item: &TokenVariance) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TokenVarianceRequiredSourceRule {
    pub message: String,
}

impl TokenVarianceRule for TokenVarianceRequiredSourceRule {
    fn check(&self, item: &TokenVariance) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a token variance.
#[derive(Debug, Clone)]
pub struct TokenVarianceValidator {
    pub rules: Vec<Rc<dyn TokenVarianceRule>>,
}

impl TokenVarianceValidator {
    pub fn validate(&self, item: &TokenVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TokenVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TokenVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_token_variance_validator() -> TokenVarianceValidator {
    TokenVarianceValidator {
        rules: vec![
            Rc::new(TokenVarianceAllowedExternalRefRule { allowed: Vec::new() }) as Rc<dyn TokenVarianceRule>,
            Rc::new(TokenVarianceAmountRangeRule { min: 7.4, max: 3684.4 }),
            Rc::new(TokenVarianceRequiredSourceRule {
                message: "token variance needs an entry".to_string(),
            }),
        ],
    }
}
