use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual trust domain score must pass. Check returns its problems, or none.
pub trait ManualTrustDomainScoreRule: std::fmt::Debug {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreRequiredCodeRule {
    pub message: String,
}

impl ManualTrustDomainScoreRule for ManualTrustDomainScoreRequiredCodeRule {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreCodePrefixRule {
    pub prefix: String,
}

impl ManualTrustDomainScoreRule for ManualTrustDomainScoreCodePrefixRule {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreLabelLengthRule {
    pub max_length: i64,
}

impl ManualTrustDomainScoreRule for ManualTrustDomainScoreLabelLengthRule {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreRequiredUnitRule {
    pub message: String,
}

impl ManualTrustDomainScoreRule for ManualTrustDomainScoreRequiredUnitRule {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreTitleLengthRule {
    pub max_length: i64,
}

impl ManualTrustDomainScoreRule for ManualTrustDomainScoreTitleLengthRule {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScoreCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualTrustDomainScoreRule for ManualTrustDomainScoreCostRangeRule {
    fn check(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual trust domain score.
#[derive(Debug, Clone)]
pub struct ManualTrustDomainScoreValidator {
    pub rules: Vec<Rc<dyn ManualTrustDomainScoreRule>>,
}

impl ManualTrustDomainScoreValidator {
    pub fn validate(&self, item: &ManualTrustDomainScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualTrustDomainScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTrustDomainScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_trust_domain_score_validator() -> ManualTrustDomainScoreValidator {
    ManualTrustDomainScoreValidator {
        rules: vec![
            Rc::new(ManualTrustDomainScoreRequiredCodeRule {
                message: "manual trust domain score needs this field".to_string(),
            }) as Rc<dyn ManualTrustDomainScoreRule>,
            Rc::new(ManualTrustDomainScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ManualTrustDomainScoreLabelLengthRule { max_length: 64 }),
            Rc::new(ManualTrustDomainScoreRequiredUnitRule {
                message: "manual trust domain score needs a value".to_string(),
            }),
            Rc::new(ManualTrustDomainScoreTitleLengthRule { max_length: 80 }),
            Rc::new(ManualTrustDomainScoreCostRangeRule { min: 4.7, max: 904.7 }),
        ],
    }
}
