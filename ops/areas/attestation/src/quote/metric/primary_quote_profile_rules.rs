use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary quote profile must pass. Check returns its problems, or none.
pub trait PrimaryQuoteProfileRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfileCodePrefixRule {
    pub prefix: String,
}

impl PrimaryQuoteProfileRule for PrimaryQuoteProfileCodePrefixRule {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfilePrincipalLengthRule {
    pub max_length: i64,
}

impl PrimaryQuoteProfileRule for PrimaryQuoteProfilePrincipalLengthRule {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfileRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryQuoteProfileRule for PrimaryQuoteProfileRateRangeRule {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfileVisibleGuardRule {
    pub limit: f64,
}

impl PrimaryQuoteProfileRule for PrimaryQuoteProfileVisibleGuardRule {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        if item.visible && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while visible", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfileWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryQuoteProfileRule for PrimaryQuoteProfileWeightRangeRule {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryQuoteProfileRequiredLabelRule {
    pub message: String,
}

impl PrimaryQuoteProfileRule for PrimaryQuoteProfileRequiredLabelRule {
    fn check(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary quote profile.
#[derive(Debug, Clone)]
pub struct PrimaryQuoteProfileValidator {
    pub rules: Vec<Rc<dyn PrimaryQuoteProfileRule>>,
}

impl PrimaryQuoteProfileValidator {
    pub fn validate(&self, item: &PrimaryQuoteProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryQuoteProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryQuoteProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_quote_profile_validator() -> PrimaryQuoteProfileValidator {
    PrimaryQuoteProfileValidator {
        rules: vec![
            Rc::new(PrimaryQuoteProfileCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn PrimaryQuoteProfileRule>,
            Rc::new(PrimaryQuoteProfilePrincipalLengthRule { max_length: 80 }),
            Rc::new(PrimaryQuoteProfileRateRangeRule { min: 5.2, max: 96.9 }),
            Rc::new(PrimaryQuoteProfileVisibleGuardRule { limit: 2.5 }),
            Rc::new(PrimaryQuoteProfileWeightRangeRule { min: 8.1, max: 395.9 }),
            Rc::new(PrimaryQuoteProfileRequiredLabelRule {
                message: "primary quote profile needs an entry".to_string(),
            }),
        ],
    }
}
