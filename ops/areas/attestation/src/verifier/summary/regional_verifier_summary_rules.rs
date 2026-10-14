use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional verifier summary must pass. Check returns its problems, or none.
pub trait RegionalVerifierSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RegionalVerifierSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSummaryCodeLengthRule {
    pub max_length: i64,
}

impl RegionalVerifierSummaryRule for RegionalVerifierSummaryCodeLengthRule {
    fn check(&self, item: &RegionalVerifierSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSummaryScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalVerifierSummaryRule for RegionalVerifierSummaryScoreRangeRule {
    fn check(&self, item: &RegionalVerifierSummary) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSummaryCipherLengthRule {
    pub max_length: i64,
}

impl RegionalVerifierSummaryRule for RegionalVerifierSummaryCipherLengthRule {
    fn check(&self, item: &RegionalVerifierSummary) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSummaryAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RegionalVerifierSummaryRule for RegionalVerifierSummaryAllowedNameRule {
    fn check(&self, item: &RegionalVerifierSummary) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional verifier summary.
#[derive(Debug, Clone)]
pub struct RegionalVerifierSummaryValidator {
    pub rules: Vec<Rc<dyn RegionalVerifierSummaryRule>>,
}

impl RegionalVerifierSummaryValidator {
    pub fn validate(&self, item: &RegionalVerifierSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalVerifierSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalVerifierSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_verifier_summary_validator() -> RegionalVerifierSummaryValidator {
    RegionalVerifierSummaryValidator {
        rules: vec![
            Rc::new(RegionalVerifierSummaryCodeLengthRule { max_length: 86400 }) as Rc<dyn RegionalVerifierSummaryRule>,
            Rc::new(RegionalVerifierSummaryScoreRangeRule { min: 7.6, max: 3099.2 }),
            Rc::new(RegionalVerifierSummaryCipherLengthRule { max_length: 86400 }),
            Rc::new(RegionalVerifierSummaryAllowedNameRule {
                allowed: vec!["capped".to_string(), "weighted".to_string()],
            }),
        ],
    }
}
