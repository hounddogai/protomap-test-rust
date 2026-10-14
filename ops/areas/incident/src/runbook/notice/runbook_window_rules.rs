use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook window must pass. Check returns its problems, or none.
pub trait RunbookWindowRule: std::fmt::Debug {
    fn check(&self, item: &RunbookWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindowRequiredCodeRule {
    pub message: String,
}

impl RunbookWindowRule for RunbookWindowRequiredCodeRule {
    fn check(&self, item: &RunbookWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindowCodeLengthRule {
    pub max_length: i64,
}

impl RunbookWindowRule for RunbookWindowCodeLengthRule {
    fn check(&self, item: &RunbookWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindowCountryLengthRule {
    pub max_length: i64,
}

impl RunbookWindowRule for RunbookWindowCountryLengthRule {
    fn check(&self, item: &RunbookWindow) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindowLabelLengthRule {
    pub max_length: i64,
}

impl RunbookWindowRule for RunbookWindowLabelLengthRule {
    fn check(&self, item: &RunbookWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindowRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookWindowRule for RunbookWindowRatioRangeRule {
    fn check(&self, item: &RunbookWindow) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindowAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl RunbookWindowRule for RunbookWindowAllowedCountryRule {
    fn check(&self, item: &RunbookWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is above maximum", item.country)];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook window.
#[derive(Debug, Clone)]
pub struct RunbookWindowValidator {
    pub rules: Vec<Rc<dyn RunbookWindowRule>>,
}

impl RunbookWindowValidator {
    pub fn validate(&self, item: &RunbookWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_window_validator() -> RunbookWindowValidator {
    RunbookWindowValidator {
        rules: vec![
            Rc::new(RunbookWindowRequiredCodeRule {
                message: "runbook window needs this field".to_string(),
            }) as Rc<dyn RunbookWindowRule>,
            Rc::new(RunbookWindowCodeLengthRule { max_length: 3600 }),
            Rc::new(RunbookWindowCountryLengthRule { max_length: 1000 }),
            Rc::new(RunbookWindowLabelLengthRule { max_length: 16 }),
            Rc::new(RunbookWindowRatioRangeRule { min: 0.3, max: 3832.2 }),
            Rc::new(RunbookWindowAllowedCountryRule { allowed: Vec::new() }),
        ],
    }
}
