use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal postmortem rollup must pass. Check returns its problems, or none.
pub trait SeasonalPostmortemRollupRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalPostmortemRollupRule for SeasonalPostmortemRollupLabelLengthRule {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SeasonalPostmortemRollupRule for SeasonalPostmortemRollupAllowedTitleRule {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupRequiredCodeRule {
    pub message: String,
}

impl SeasonalPostmortemRollupRule for SeasonalPostmortemRollupRequiredCodeRule {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalPostmortemRollupRule for SeasonalPostmortemRollupTitleLengthRule {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalPostmortemRollupRule for SeasonalPostmortemRollupCodeLengthRule {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPostmortemRollupRequiredTitleRule {
    pub message: String,
}

impl SeasonalPostmortemRollupRule for SeasonalPostmortemRollupRequiredTitleRule {
    fn check(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal postmortem rollup.
#[derive(Debug, Clone)]
pub struct SeasonalPostmortemRollupValidator {
    pub rules: Vec<Rc<dyn SeasonalPostmortemRollupRule>>,
}

impl SeasonalPostmortemRollupValidator {
    pub fn validate(&self, item: &SeasonalPostmortemRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalPostmortemRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPostmortemRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_postmortem_rollup_validator() -> SeasonalPostmortemRollupValidator {
    SeasonalPostmortemRollupValidator {
        rules: vec![
            Rc::new(SeasonalPostmortemRollupLabelLengthRule { max_length: 80 }) as Rc<dyn SeasonalPostmortemRollupRule>,
            Rc::new(SeasonalPostmortemRollupAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(SeasonalPostmortemRollupRequiredCodeRule {
                message: "seasonal postmortem rollup needs a value".to_string(),
            }),
            Rc::new(SeasonalPostmortemRollupTitleLengthRule { max_length: 32 }),
            Rc::new(SeasonalPostmortemRollupCodeLengthRule { max_length: 3600 }),
            Rc::new(SeasonalPostmortemRollupRequiredTitleRule {
                message: "seasonal postmortem rollup needs a value".to_string(),
            }),
        ],
    }
}
