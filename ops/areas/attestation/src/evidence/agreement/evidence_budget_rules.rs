use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a evidence budget must pass. Check returns its problems, or none.
pub trait EvidenceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBudgetRequiredCodeRule {
    pub message: String,
}

impl EvidenceBudgetRule for EvidenceBudgetRequiredCodeRule {
    fn check(&self, item: &EvidenceBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBudgetRequiredLabelRule {
    pub message: String,
}

impl EvidenceBudgetRule for EvidenceBudgetRequiredLabelRule {
    fn check(&self, item: &EvidenceBudget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceBudgetRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceBudgetRule for EvidenceBudgetRotationDaysRangeRule {
    fn check(&self, item: &EvidenceBudget) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a evidence budget.
#[derive(Debug, Clone)]
pub struct EvidenceBudgetValidator {
    pub rules: Vec<Rc<dyn EvidenceBudgetRule>>,
}

impl EvidenceBudgetValidator {
    pub fn validate(&self, item: &EvidenceBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EvidenceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_budget_validator() -> EvidenceBudgetValidator {
    EvidenceBudgetValidator {
        rules: vec![
            Rc::new(EvidenceBudgetRequiredCodeRule {
                message: "evidence budget needs a value".to_string(),
            }) as Rc<dyn EvidenceBudgetRule>,
            Rc::new(EvidenceBudgetRequiredLabelRule {
                message: "evidence budget needs an entry".to_string(),
            }),
            Rc::new(EvidenceBudgetRotationDaysRangeRule { min: 3600, max: 3600 }),
        ],
    }
}
