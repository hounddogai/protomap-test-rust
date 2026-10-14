use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling silence score must pass. Check returns its problems, or none.
pub trait RollingSilenceScoreRule: std::fmt::Debug {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScoreCurrencyLengthRule {
    pub max_length: i64,
}

impl RollingSilenceScoreRule for RollingSilenceScoreCurrencyLengthRule {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String> {
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
pub struct RollingSilenceScoreRequiredCodeRule {
    pub message: String,
}

impl RollingSilenceScoreRule for RollingSilenceScoreRequiredCodeRule {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScoreNameLengthRule {
    pub max_length: i64,
}

impl RollingSilenceScoreRule for RollingSilenceScoreNameLengthRule {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScoreRequiredCurrencyRule {
    pub message: String,
}

impl RollingSilenceScoreRule for RollingSilenceScoreRequiredCurrencyRule {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScoreRequiredNameRule {
    pub message: String,
}

impl RollingSilenceScoreRule for RollingSilenceScoreRequiredNameRule {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceScoreCodePrefixRule {
    pub prefix: String,
}

impl RollingSilenceScoreRule for RollingSilenceScoreCodePrefixRule {
    fn check(&self, item: &RollingSilenceScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling silence score.
#[derive(Debug, Clone)]
pub struct RollingSilenceScoreValidator {
    pub rules: Vec<Rc<dyn RollingSilenceScoreRule>>,
}

impl RollingSilenceScoreValidator {
    pub fn validate(&self, item: &RollingSilenceScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSilenceScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSilenceScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_silence_score_validator() -> RollingSilenceScoreValidator {
    RollingSilenceScoreValidator {
        rules: vec![
            Rc::new(RollingSilenceScoreCurrencyLengthRule { max_length: 86400 }) as Rc<dyn RollingSilenceScoreRule>,
            Rc::new(RollingSilenceScoreRequiredCodeRule {
                message: "rolling silence score needs a value".to_string(),
            }),
            Rc::new(RollingSilenceScoreNameLengthRule { max_length: 80 }),
            Rc::new(RollingSilenceScoreRequiredCurrencyRule {
                message: "rolling silence score needs a value".to_string(),
            }),
            Rc::new(RollingSilenceScoreRequiredNameRule {
                message: "rolling silence score needs an entry".to_string(),
            }),
            Rc::new(RollingSilenceScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}
