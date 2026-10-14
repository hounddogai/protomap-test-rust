use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction override must pass. Check returns its problems, or none.
pub trait RedactionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &RedactionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionOverrideQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionOverrideRule for RedactionOverrideQuotaUnitsRangeRule {
    fn check(&self, item: &RedactionOverride) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionOverrideRule for RedactionOverrideStartsAtRangeRule {
    fn check(&self, item: &RedactionOverride) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionOverrideSummaryLengthRule {
    pub max_length: i64,
}

impl RedactionOverrideRule for RedactionOverrideSummaryLengthRule {
    fn check(&self, item: &RedactionOverride) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionOverrideCodeLengthRule {
    pub max_length: i64,
}

impl RedactionOverrideRule for RedactionOverrideCodeLengthRule {
    fn check(&self, item: &RedactionOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction override.
#[derive(Debug, Clone)]
pub struct RedactionOverrideValidator {
    pub rules: Vec<Rc<dyn RedactionOverrideRule>>,
}

impl RedactionOverrideValidator {
    pub fn validate(&self, item: &RedactionOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_override_validator() -> RedactionOverrideValidator {
    RedactionOverrideValidator {
        rules: vec![
            Rc::new(RedactionOverrideQuotaUnitsRangeRule { min: 32, max: 64 }) as Rc<dyn RedactionOverrideRule>,
            Rc::new(RedactionOverrideStartsAtRangeRule { min: 64, max: 80 }),
            Rc::new(RedactionOverrideSummaryLengthRule { max_length: 80 }),
            Rc::new(RedactionOverrideCodeLengthRule { max_length: 86400 }),
        ],
    }
}
