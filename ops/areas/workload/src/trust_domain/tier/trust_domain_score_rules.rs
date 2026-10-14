use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a trust domain score must pass. Check returns its problems, or none.
pub trait TrustDomainScoreRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScoreQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainScoreRule for TrustDomainScoreQuantityRangeRule {
    fn check(&self, item: &TrustDomainScore) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScoreLabelLengthRule {
    pub max_length: i64,
}

impl TrustDomainScoreRule for TrustDomainScoreLabelLengthRule {
    fn check(&self, item: &TrustDomainScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScoreAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TrustDomainScoreRule for TrustDomainScoreAllowedCodeRule {
    fn check(&self, item: &TrustDomainScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainScoreRequiredLabelRule {
    pub message: String,
}

impl TrustDomainScoreRule for TrustDomainScoreRequiredLabelRule {
    fn check(&self, item: &TrustDomainScore) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain score.
#[derive(Debug, Clone)]
pub struct TrustDomainScoreValidator {
    pub rules: Vec<Rc<dyn TrustDomainScoreRule>>,
}

impl TrustDomainScoreValidator {
    pub fn validate(&self, item: &TrustDomainScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TrustDomainScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_score_validator() -> TrustDomainScoreValidator {
    TrustDomainScoreValidator {
        rules: vec![
            Rc::new(TrustDomainScoreQuantityRangeRule { min: 32, max: 80 }) as Rc<dyn TrustDomainScoreRule>,
            Rc::new(TrustDomainScoreLabelLengthRule { max_length: 3600 }),
            Rc::new(TrustDomainScoreAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(TrustDomainScoreRequiredLabelRule {
                message: "trust domain score needs an entry".to_string(),
            }),
        ],
    }
}
