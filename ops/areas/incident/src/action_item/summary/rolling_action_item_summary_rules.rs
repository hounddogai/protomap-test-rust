use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling action item summary must pass. Check returns its problems, or none.
pub trait RollingActionItemSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingActionItemSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingActionItemSummaryRequiredCountryRule {
    pub message: String,
}

impl RollingActionItemSummaryRule for RollingActionItemSummaryRequiredCountryRule {
    fn check(&self, item: &RollingActionItemSummary) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingActionItemSummaryRequiredCodeRule {
    pub message: String,
}

impl RollingActionItemSummaryRule for RollingActionItemSummaryRequiredCodeRule {
    fn check(&self, item: &RollingActionItemSummary) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingActionItemSummaryStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingActionItemSummaryRule for RollingActionItemSummaryStartsAtRangeRule {
    fn check(&self, item: &RollingActionItemSummary) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingActionItemSummaryCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingActionItemSummaryRule for RollingActionItemSummaryCeilingRangeRule {
    fn check(&self, item: &RollingActionItemSummary) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingActionItemSummaryTitleLengthRule {
    pub max_length: i64,
}

impl RollingActionItemSummaryRule for RollingActionItemSummaryTitleLengthRule {
    fn check(&self, item: &RollingActionItemSummary) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling action item summary.
#[derive(Debug, Clone)]
pub struct RollingActionItemSummaryValidator {
    pub rules: Vec<Rc<dyn RollingActionItemSummaryRule>>,
}

impl RollingActionItemSummaryValidator {
    pub fn validate(&self, item: &RollingActionItemSummary) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingActionItemSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingActionItemSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_action_item_summary_validator() -> RollingActionItemSummaryValidator {
    RollingActionItemSummaryValidator {
        rules: vec![
            Rc::new(RollingActionItemSummaryRequiredCountryRule {
                message: "rolling action item summary needs a value".to_string(),
            }) as Rc<dyn RollingActionItemSummaryRule>,
            Rc::new(RollingActionItemSummaryRequiredCodeRule {
                message: "rolling action item summary needs a value".to_string(),
            }),
            Rc::new(RollingActionItemSummaryStartsAtRangeRule { min: 80, max: 255 }),
            Rc::new(RollingActionItemSummaryCeilingRangeRule { min: 8.8, max: 3138.2 }),
            Rc::new(RollingActionItemSummaryTitleLengthRule { max_length: 120 }),
        ],
    }
}
