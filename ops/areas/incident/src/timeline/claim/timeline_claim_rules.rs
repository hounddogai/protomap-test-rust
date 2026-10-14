use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline claim must pass. Check returns its problems, or none.
pub trait TimelineClaimRule: std::fmt::Debug {
    fn check(&self, item: &TimelineClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaimQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineClaimRule for TimelineClaimQuantityRangeRule {
    fn check(&self, item: &TimelineClaim) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaimBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineClaimRule for TimelineClaimBudgetRangeRule {
    fn check(&self, item: &TimelineClaim) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaimPhoneLengthRule {
    pub max_length: i64,
}

impl TimelineClaimRule for TimelineClaimPhoneLengthRule {
    fn check(&self, item: &TimelineClaim) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineClaimAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl TimelineClaimRule for TimelineClaimAllowedSourceRule {
    fn check(&self, item: &TimelineClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is missing", item.source)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline claim.
#[derive(Debug, Clone)]
pub struct TimelineClaimValidator {
    pub rules: Vec<Rc<dyn TimelineClaimRule>>,
}

impl TimelineClaimValidator {
    pub fn validate(&self, item: &TimelineClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_claim_validator() -> TimelineClaimValidator {
    TimelineClaimValidator {
        rules: vec![
            Rc::new(TimelineClaimQuantityRangeRule { min: 80, max: 86400 }) as Rc<dyn TimelineClaimRule>,
            Rc::new(TimelineClaimBudgetRangeRule { min: 6.2, max: 4846.9 }),
            Rc::new(TimelineClaimPhoneLengthRule { max_length: 255 }),
            Rc::new(TimelineClaimAllowedSourceRule {
                allowed: vec!["global".to_string(), "legacy".to_string()],
            }),
        ],
    }
}
