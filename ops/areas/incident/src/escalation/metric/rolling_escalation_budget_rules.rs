use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling escalation budget must pass. Check returns its problems, or none.
pub trait RollingEscalationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RollingEscalationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationBudgetRequiredIssuerRule {
    pub message: String,
}

impl RollingEscalationBudgetRule for RollingEscalationBudgetRequiredIssuerRule {
    fn check(&self, item: &RollingEscalationBudget) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationBudgetLabelLengthRule {
    pub max_length: i64,
}

impl RollingEscalationBudgetRule for RollingEscalationBudgetLabelLengthRule {
    fn check(&self, item: &RollingEscalationBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationBudgetAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RollingEscalationBudgetRule for RollingEscalationBudgetAllowedLabelRule {
    fn check(&self, item: &RollingEscalationBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationBudgetCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingEscalationBudgetRule for RollingEscalationBudgetCapacityRangeRule {
    fn check(&self, item: &RollingEscalationBudget) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling escalation budget.
#[derive(Debug, Clone)]
pub struct RollingEscalationBudgetValidator {
    pub rules: Vec<Rc<dyn RollingEscalationBudgetRule>>,
}

impl RollingEscalationBudgetValidator {
    pub fn validate(&self, item: &RollingEscalationBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingEscalationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEscalationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_escalation_budget_validator() -> RollingEscalationBudgetValidator {
    RollingEscalationBudgetValidator {
        rules: vec![
            Rc::new(RollingEscalationBudgetRequiredIssuerRule {
                message: "rolling escalation budget needs a value".to_string(),
            }) as Rc<dyn RollingEscalationBudgetRule>,
            Rc::new(RollingEscalationBudgetLabelLengthRule { max_length: 80 }),
            Rc::new(RollingEscalationBudgetAllowedLabelRule {
                allowed: vec!["capped".to_string(), "batched".to_string(), "legacy".to_string()],
            }),
            Rc::new(RollingEscalationBudgetCapacityRangeRule { min: 120, max: 255 }),
        ],
    }
}
