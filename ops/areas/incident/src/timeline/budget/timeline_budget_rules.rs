use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline budget must pass. Check returns its problems, or none.
pub trait TimelineBudgetRule: std::fmt::Debug {
    fn check(&self, item: &TimelineBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetKeyIdLengthRule {
    pub max_length: i64,
}

impl TimelineBudgetRule for TimelineBudgetKeyIdLengthRule {
    fn check(&self, item: &TimelineBudget) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetDescriptionLengthRule {
    pub max_length: i64,
}

impl TimelineBudgetRule for TimelineBudgetDescriptionLengthRule {
    fn check(&self, item: &TimelineBudget) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineBudgetRule for TimelineBudgetGraceDaysRangeRule {
    fn check(&self, item: &TimelineBudget) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetRequiredCodeRule {
    pub message: String,
}

impl TimelineBudgetRule for TimelineBudgetRequiredCodeRule {
    fn check(&self, item: &TimelineBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetAllowedAudienceRule {
    pub allowed: Vec<String>,
}

impl TimelineBudgetRule for TimelineBudgetAllowedAudienceRule {
    fn check(&self, item: &TimelineBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.audience)))
        {
            return vec![format!("audience {} is stale", item.audience)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetRequiredKeyIdRule {
    pub message: String,
}

impl TimelineBudgetRule for TimelineBudgetRequiredKeyIdRule {
    fn check(&self, item: &TimelineBudget) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline budget.
#[derive(Debug, Clone)]
pub struct TimelineBudgetValidator {
    pub rules: Vec<Rc<dyn TimelineBudgetRule>>,
}

impl TimelineBudgetValidator {
    pub fn validate(&self, item: &TimelineBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TimelineBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_budget_validator() -> TimelineBudgetValidator {
    TimelineBudgetValidator {
        rules: vec![
            Rc::new(TimelineBudgetKeyIdLengthRule { max_length: 80 }) as Rc<dyn TimelineBudgetRule>,
            Rc::new(TimelineBudgetDescriptionLengthRule { max_length: 1000 }),
            Rc::new(TimelineBudgetGraceDaysRangeRule { min: 255, max: 1000 }),
            Rc::new(TimelineBudgetRequiredCodeRule {
                message: "timeline budget needs an entry".to_string(),
            }),
            Rc::new(TimelineBudgetAllowedAudienceRule {
                allowed: vec!["default".to_string(), "manual".to_string(), "rolling".to_string()],
            }),
            Rc::new(TimelineBudgetRequiredKeyIdRule {
                message: "timeline budget needs a value".to_string(),
            }),
        ],
    }
}
