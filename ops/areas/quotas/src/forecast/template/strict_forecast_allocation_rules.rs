use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict forecast allocation must pass. Check returns its problems, or none.
pub trait StrictForecastAllocationRule: std::fmt::Debug {
    fn check(&self, item: &StrictForecastAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationExternalRefLengthRule {
    pub max_length: i64,
}

impl StrictForecastAllocationRule for StrictForecastAllocationExternalRefLengthRule {
    fn check(&self, item: &StrictForecastAllocation) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictForecastAllocationRule for StrictForecastAllocationGraceDaysRangeRule {
    fn check(&self, item: &StrictForecastAllocation) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictForecastAllocationRule for StrictForecastAllocationMarginRangeRule {
    fn check(&self, item: &StrictForecastAllocation) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationReviewerLengthRule {
    pub max_length: i64,
}

impl StrictForecastAllocationRule for StrictForecastAllocationReviewerLengthRule {
    fn check(&self, item: &StrictForecastAllocation) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictForecastAllocationTitleLengthRule {
    pub max_length: i64,
}

impl StrictForecastAllocationRule for StrictForecastAllocationTitleLengthRule {
    fn check(&self, item: &StrictForecastAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict forecast allocation.
#[derive(Debug, Clone)]
pub struct StrictForecastAllocationValidator {
    pub rules: Vec<Rc<dyn StrictForecastAllocationRule>>,
}

impl StrictForecastAllocationValidator {
    pub fn validate(&self, item: &StrictForecastAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictForecastAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictForecastAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_forecast_allocation_validator() -> StrictForecastAllocationValidator {
    StrictForecastAllocationValidator {
        rules: vec![
            Rc::new(StrictForecastAllocationExternalRefLengthRule { max_length: 80 })
                as Rc<dyn StrictForecastAllocationRule>,
            Rc::new(StrictForecastAllocationGraceDaysRangeRule { min: 255, max: 255 }),
            Rc::new(StrictForecastAllocationMarginRangeRule { min: 0.1, max: 4708.4 }),
            Rc::new(StrictForecastAllocationReviewerLengthRule { max_length: 3600 }),
            Rc::new(StrictForecastAllocationTitleLengthRule { max_length: 1000 }),
        ],
    }
}
