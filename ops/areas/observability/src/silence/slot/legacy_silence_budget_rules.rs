use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy silence budget must pass. Check returns its problems, or none.
pub trait LegacySilenceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LegacySilenceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceBudgetRequiredCodeRule {
    pub message: String,
}

impl LegacySilenceBudgetRule for LegacySilenceBudgetRequiredCodeRule {
    fn check(&self, item: &LegacySilenceBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceBudgetRequiredLabelRule {
    pub message: String,
}

impl LegacySilenceBudgetRule for LegacySilenceBudgetRequiredLabelRule {
    fn check(&self, item: &LegacySilenceBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceBudgetCodeLengthRule {
    pub max_length: i64,
}

impl LegacySilenceBudgetRule for LegacySilenceBudgetCodeLengthRule {
    fn check(&self, item: &LegacySilenceBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy silence budget.
#[derive(Debug, Clone)]
pub struct LegacySilenceBudgetValidator {
    pub rules: Vec<Rc<dyn LegacySilenceBudgetRule>>,
}

impl LegacySilenceBudgetValidator {
    pub fn validate(&self, item: &LegacySilenceBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacySilenceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacySilenceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_silence_budget_validator() -> LegacySilenceBudgetValidator {
    LegacySilenceBudgetValidator {
        rules: vec![
            Rc::new(LegacySilenceBudgetRequiredCodeRule {
                message: "legacy silence budget needs a value".to_string(),
            }) as Rc<dyn LegacySilenceBudgetRule>,
            Rc::new(LegacySilenceBudgetRequiredLabelRule {
                message: "legacy silence budget needs this field".to_string(),
            }),
            Rc::new(LegacySilenceBudgetCodeLengthRule { max_length: 86400 }),
        ],
    }
}
