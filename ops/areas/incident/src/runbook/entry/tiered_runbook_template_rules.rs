use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered runbook template must pass. Check returns its problems, or none.
pub trait TieredRunbookTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TieredRunbookTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookTemplateTimezoneLengthRule {
    pub max_length: i64,
}

impl TieredRunbookTemplateRule for TieredRunbookTemplateTimezoneLengthRule {
    fn check(&self, item: &TieredRunbookTemplate) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookTemplateRequiredTimezoneRule {
    pub message: String,
}

impl TieredRunbookTemplateRule for TieredRunbookTemplateRequiredTimezoneRule {
    fn check(&self, item: &TieredRunbookTemplate) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookTemplateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredRunbookTemplateRule for TieredRunbookTemplateScoreRangeRule {
    fn check(&self, item: &TieredRunbookTemplate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered runbook template.
#[derive(Debug, Clone)]
pub struct TieredRunbookTemplateValidator {
    pub rules: Vec<Rc<dyn TieredRunbookTemplateRule>>,
}

impl TieredRunbookTemplateValidator {
    pub fn validate(&self, item: &TieredRunbookTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredRunbookTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredRunbookTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_runbook_template_validator() -> TieredRunbookTemplateValidator {
    TieredRunbookTemplateValidator {
        rules: vec![
            Rc::new(TieredRunbookTemplateTimezoneLengthRule { max_length: 32 }) as Rc<dyn TieredRunbookTemplateRule>,
            Rc::new(TieredRunbookTemplateRequiredTimezoneRule {
                message: "tiered runbook template needs a value".to_string(),
            }),
            Rc::new(TieredRunbookTemplateScoreRangeRule { min: 9.9, max: 4832.1 }),
        ],
    }
}
