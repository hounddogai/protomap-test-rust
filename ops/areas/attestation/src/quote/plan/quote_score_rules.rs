use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote score must pass. Check returns its problems, or none.
pub trait QuoteScoreRule: std::fmt::Debug {
    fn check(&self, item: &QuoteScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreRequiredHostRule {
    pub message: String,
}

impl QuoteScoreRule for QuoteScoreRequiredHostRule {
    fn check(&self, item: &QuoteScore) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreCodeLengthRule {
    pub max_length: i64,
}

impl QuoteScoreRule for QuoteScoreCodeLengthRule {
    fn check(&self, item: &QuoteScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreRequiredPhoneRule {
    pub message: String,
}

impl QuoteScoreRule for QuoteScoreRequiredPhoneRule {
    fn check(&self, item: &QuoteScore) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreCodePrefixRule {
    pub prefix: String,
}

impl QuoteScoreRule for QuoteScoreCodePrefixRule {
    fn check(&self, item: &QuoteScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScoreDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteScoreRule for QuoteScoreDiscountRangeRule {
    fn check(&self, item: &QuoteScore) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote score.
#[derive(Debug, Clone)]
pub struct QuoteScoreValidator {
    pub rules: Vec<Rc<dyn QuoteScoreRule>>,
}

impl QuoteScoreValidator {
    pub fn validate(&self, item: &QuoteScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_score_validator() -> QuoteScoreValidator {
    QuoteScoreValidator {
        rules: vec![
            Rc::new(QuoteScoreRequiredHostRule {
                message: "quote score needs this field".to_string(),
            }) as Rc<dyn QuoteScoreRule>,
            Rc::new(QuoteScoreCodeLengthRule { max_length: 120 }),
            Rc::new(QuoteScoreRequiredPhoneRule {
                message: "quote score needs an entry".to_string(),
            }),
            Rc::new(QuoteScoreCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(QuoteScoreDiscountRangeRule { min: 1.0, max: 1055.9 }),
        ],
    }
}
