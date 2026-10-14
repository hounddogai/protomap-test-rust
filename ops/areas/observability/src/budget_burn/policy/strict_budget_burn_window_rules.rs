use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict budget burn window must pass. Check returns its problems, or none.
pub trait StrictBudgetBurnWindowRule: std::fmt::Debug {
    fn check(&self, item: &StrictBudgetBurnWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnWindowRequiredCodeRule {
    pub message: String,
}

impl StrictBudgetBurnWindowRule for StrictBudgetBurnWindowRequiredCodeRule {
    fn check(&self, item: &StrictBudgetBurnWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnWindowTitleLengthRule {
    pub max_length: i64,
}

impl StrictBudgetBurnWindowRule for StrictBudgetBurnWindowTitleLengthRule {
    fn check(&self, item: &StrictBudgetBurnWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnWindowCodeLengthRule {
    pub max_length: i64,
}

impl StrictBudgetBurnWindowRule for StrictBudgetBurnWindowCodeLengthRule {
    fn check(&self, item: &StrictBudgetBurnWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict budget burn window.
#[derive(Debug, Clone)]
pub struct StrictBudgetBurnWindowValidator {
    pub rules: Vec<Rc<dyn StrictBudgetBurnWindowRule>>,
}

impl StrictBudgetBurnWindowValidator {
    pub fn validate(&self, item: &StrictBudgetBurnWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictBudgetBurnWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictBudgetBurnWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_budget_burn_window_validator() -> StrictBudgetBurnWindowValidator {
    StrictBudgetBurnWindowValidator {
        rules: vec![
            Rc::new(StrictBudgetBurnWindowRequiredCodeRule {
                message: "strict budget burn window needs this field".to_string(),
            }) as Rc<dyn StrictBudgetBurnWindowRule>,
            Rc::new(StrictBudgetBurnWindowTitleLengthRule { max_length: 3600 }),
            Rc::new(StrictBudgetBurnWindowCodeLengthRule { max_length: 120 }),
        ],
    }
}
