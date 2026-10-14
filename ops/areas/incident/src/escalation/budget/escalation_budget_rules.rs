use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation budget must pass. Check returns its problems, or none.
pub trait EscalationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &EscalationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetAudienceLengthRule {
    pub max_length: i64,
}

impl EscalationBudgetRule for EscalationBudgetAudienceLengthRule {
    fn check(&self, item: &EscalationBudget) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetRequiredTitleRule {
    pub message: String,
}

impl EscalationBudgetRule for EscalationBudgetRequiredTitleRule {
    fn check(&self, item: &EscalationBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetCurrencyLengthRule {
    pub max_length: i64,
}

impl EscalationBudgetRule for EscalationBudgetCurrencyLengthRule {
    fn check(&self, item: &EscalationBudget) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl EscalationBudgetRule for EscalationBudgetAllowedTitleRule {
    fn check(&self, item: &EscalationBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetTitleLengthRule {
    pub max_length: i64,
}

impl EscalationBudgetRule for EscalationBudgetTitleLengthRule {
    fn check(&self, item: &EscalationBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetRequiredCurrencyRule {
    pub message: String,
}

impl EscalationBudgetRule for EscalationBudgetRequiredCurrencyRule {
    fn check(&self, item: &EscalationBudget) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation budget.
#[derive(Debug, Clone)]
pub struct EscalationBudgetValidator {
    pub rules: Vec<Rc<dyn EscalationBudgetRule>>,
}

impl EscalationBudgetValidator {
    pub fn validate(&self, item: &EscalationBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_budget_validator() -> EscalationBudgetValidator {
    EscalationBudgetValidator {
        rules: vec![
            Rc::new(EscalationBudgetAudienceLengthRule { max_length: 1000 }) as Rc<dyn EscalationBudgetRule>,
            Rc::new(EscalationBudgetRequiredTitleRule {
                message: "escalation budget needs an entry".to_string(),
            }),
            Rc::new(EscalationBudgetCurrencyLengthRule { max_length: 16 }),
            Rc::new(EscalationBudgetAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "seasonal".to_string(), "legacy".to_string()],
            }),
            Rc::new(EscalationBudgetTitleLengthRule { max_length: 80 }),
            Rc::new(EscalationBudgetRequiredCurrencyRule {
                message: "escalation budget needs a value".to_string(),
            }),
        ],
    }
}
