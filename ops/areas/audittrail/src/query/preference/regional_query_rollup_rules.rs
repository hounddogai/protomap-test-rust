use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional query rollup must pass. Check returns its problems, or none.
pub trait RegionalQueryRollupRule: std::fmt::Debug {
    fn check(&self, item: &RegionalQueryRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionalQueryRollupRule for RegionalQueryRollupAllowedTitleRule {
    fn check(&self, item: &RegionalQueryRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalQueryRollupRule for RegionalQueryRollupGraceDaysRangeRule {
    fn check(&self, item: &RegionalQueryRollup) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollupTitleLengthRule {
    pub max_length: i64,
}

impl RegionalQueryRollupRule for RegionalQueryRollupTitleLengthRule {
    fn check(&self, item: &RegionalQueryRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollupRequiredCodeRule {
    pub message: String,
}

impl RegionalQueryRollupRule for RegionalQueryRollupRequiredCodeRule {
    fn check(&self, item: &RegionalQueryRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryRollupRequiredTitleRule {
    pub message: String,
}

impl RegionalQueryRollupRule for RegionalQueryRollupRequiredTitleRule {
    fn check(&self, item: &RegionalQueryRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional query rollup.
#[derive(Debug, Clone)]
pub struct RegionalQueryRollupValidator {
    pub rules: Vec<Rc<dyn RegionalQueryRollupRule>>,
}

impl RegionalQueryRollupValidator {
    pub fn validate(&self, item: &RegionalQueryRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalQueryRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalQueryRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_query_rollup_validator() -> RegionalQueryRollupValidator {
    RegionalQueryRollupValidator {
        rules: vec![
            Rc::new(RegionalQueryRollupAllowedTitleRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn RegionalQueryRollupRule>,
            Rc::new(RegionalQueryRollupGraceDaysRangeRule { min: 32, max: 64 }),
            Rc::new(RegionalQueryRollupTitleLengthRule { max_length: 64 }),
            Rc::new(RegionalQueryRollupRequiredCodeRule {
                message: "regional query rollup needs an entry".to_string(),
            }),
            Rc::new(RegionalQueryRollupRequiredTitleRule {
                message: "regional query rollup needs a value".to_string(),
            }),
        ],
    }
}
