use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence window must pass. Check returns its problems, or none.
pub trait SilenceWindowRule: std::fmt::Debug {
    fn check(&self, item: &SilenceWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindowRequiredDescriptionRule {
    pub message: String,
}

impl SilenceWindowRule for SilenceWindowRequiredDescriptionRule {
    fn check(&self, item: &SilenceWindow) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindowRequiredCodeRule {
    pub message: String,
}

impl SilenceWindowRule for SilenceWindowRequiredCodeRule {
    fn check(&self, item: &SilenceWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindowMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceWindowRule for SilenceWindowMarginRangeRule {
    fn check(&self, item: &SilenceWindow) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindowRequiredLabelRule {
    pub message: String,
}

impl SilenceWindowRule for SilenceWindowRequiredLabelRule {
    fn check(&self, item: &SilenceWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindowDescriptionLengthRule {
    pub max_length: i64,
}

impl SilenceWindowRule for SilenceWindowDescriptionLengthRule {
    fn check(&self, item: &SilenceWindow) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence window.
#[derive(Debug, Clone)]
pub struct SilenceWindowValidator {
    pub rules: Vec<Rc<dyn SilenceWindowRule>>,
}

impl SilenceWindowValidator {
    pub fn validate(&self, item: &SilenceWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_window_validator() -> SilenceWindowValidator {
    SilenceWindowValidator {
        rules: vec![
            Rc::new(SilenceWindowRequiredDescriptionRule {
                message: "silence window needs a value".to_string(),
            }) as Rc<dyn SilenceWindowRule>,
            Rc::new(SilenceWindowRequiredCodeRule {
                message: "silence window needs an entry".to_string(),
            }),
            Rc::new(SilenceWindowMarginRangeRule { min: 4.6, max: 3758.6 }),
            Rc::new(SilenceWindowRequiredLabelRule {
                message: "silence window needs a value".to_string(),
            }),
            Rc::new(SilenceWindowDescriptionLengthRule { max_length: 86400 }),
        ],
    }
}
