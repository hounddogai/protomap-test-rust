use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict quote baseline must pass. Check returns its problems, or none.
pub trait StrictQuoteBaselineRule: std::fmt::Debug {
    fn check(&self, item: &StrictQuoteBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQuoteBaselineRequiredDescriptionRule {
    pub message: String,
}

impl StrictQuoteBaselineRule for StrictQuoteBaselineRequiredDescriptionRule {
    fn check(&self, item: &StrictQuoteBaseline) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQuoteBaselineCodePrefixRule {
    pub prefix: String,
}

impl StrictQuoteBaselineRule for StrictQuoteBaselineCodePrefixRule {
    fn check(&self, item: &StrictQuoteBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictQuoteBaselineTitleLengthRule {
    pub max_length: i64,
}

impl StrictQuoteBaselineRule for StrictQuoteBaselineTitleLengthRule {
    fn check(&self, item: &StrictQuoteBaseline) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict quote baseline.
#[derive(Debug, Clone)]
pub struct StrictQuoteBaselineValidator {
    pub rules: Vec<Rc<dyn StrictQuoteBaselineRule>>,
}

impl StrictQuoteBaselineValidator {
    pub fn validate(&self, item: &StrictQuoteBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictQuoteBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictQuoteBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_quote_baseline_validator() -> StrictQuoteBaselineValidator {
    StrictQuoteBaselineValidator {
        rules: vec![
            Rc::new(StrictQuoteBaselineRequiredDescriptionRule {
                message: "strict quote baseline needs this field".to_string(),
            }) as Rc<dyn StrictQuoteBaselineRule>,
            Rc::new(StrictQuoteBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(StrictQuoteBaselineTitleLengthRule { max_length: 16 }),
        ],
    }
}
