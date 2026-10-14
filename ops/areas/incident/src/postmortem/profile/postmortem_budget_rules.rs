use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a postmortem budget must pass. Check returns its problems, or none.
pub trait PostmortemBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudgetUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemBudgetRule for PostmortemBudgetUpdatedAtRangeRule {
    fn check(&self, item: &PostmortemBudget) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudgetAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl PostmortemBudgetRule for PostmortemBudgetAllowedOwnerIdRule {
    fn check(&self, item: &PostmortemBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is unknown", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudgetQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemBudgetRule for PostmortemBudgetQuotaUnitsRangeRule {
    fn check(&self, item: &PostmortemBudget) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem budget.
#[derive(Debug, Clone)]
pub struct PostmortemBudgetValidator {
    pub rules: Vec<Rc<dyn PostmortemBudgetRule>>,
}

impl PostmortemBudgetValidator {
    pub fn validate(&self, item: &PostmortemBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_budget_validator() -> PostmortemBudgetValidator {
    PostmortemBudgetValidator {
        rules: vec![
            Rc::new(PostmortemBudgetUpdatedAtRangeRule { min: 16, max: 1000 }) as Rc<dyn PostmortemBudgetRule>,
            Rc::new(PostmortemBudgetAllowedOwnerIdRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(PostmortemBudgetQuotaUnitsRangeRule { min: 80, max: 255 }),
        ],
    }
}
