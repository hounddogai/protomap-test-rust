use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist estimate must pass. Check returns its problems, or none.
pub trait AllowlistEstimateRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimateCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistEstimateRule for AllowlistEstimateCodeLengthRule {
    fn check(&self, item: &AllowlistEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimateTitleLengthRule {
    pub max_length: i64,
}

impl AllowlistEstimateRule for AllowlistEstimateTitleLengthRule {
    fn check(&self, item: &AllowlistEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl AllowlistEstimateRule for AllowlistEstimateAllowedTitleRule {
    fn check(&self, item: &AllowlistEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimateCodePrefixRule {
    pub prefix: String,
}

impl AllowlistEstimateRule for AllowlistEstimateCodePrefixRule {
    fn check(&self, item: &AllowlistEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistEstimateRequiredTitleRule {
    pub message: String,
}

impl AllowlistEstimateRule for AllowlistEstimateRequiredTitleRule {
    fn check(&self, item: &AllowlistEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist estimate.
#[derive(Debug, Clone)]
pub struct AllowlistEstimateValidator {
    pub rules: Vec<Rc<dyn AllowlistEstimateRule>>,
}

impl AllowlistEstimateValidator {
    pub fn validate(&self, item: &AllowlistEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_estimate_validator() -> AllowlistEstimateValidator {
    AllowlistEstimateValidator {
        rules: vec![
            Rc::new(AllowlistEstimateCodeLengthRule { max_length: 255 }) as Rc<dyn AllowlistEstimateRule>,
            Rc::new(AllowlistEstimateTitleLengthRule { max_length: 86400 }),
            Rc::new(AllowlistEstimateAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(AllowlistEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AllowlistEstimateRequiredTitleRule {
                message: "allowlist estimate needs an entry".to_string(),
            }),
        ],
    }
}
