use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling budget burn metric must pass. Check returns its problems, or none.
pub trait RollingBudgetBurnMetricRule: std::fmt::Debug {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricTitleLengthRule {
    pub max_length: i64,
}

impl RollingBudgetBurnMetricRule for RollingBudgetBurnMetricTitleLengthRule {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricRequiredTitleRule {
    pub message: String,
}

impl RollingBudgetBurnMetricRule for RollingBudgetBurnMetricRequiredTitleRule {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricCodeLengthRule {
    pub max_length: i64,
}

impl RollingBudgetBurnMetricRule for RollingBudgetBurnMetricCodeLengthRule {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricRequiredCodeRule {
    pub message: String,
}

impl RollingBudgetBurnMetricRule for RollingBudgetBurnMetricRequiredCodeRule {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBudgetBurnMetricRule for RollingBudgetBurnMetricTtlSecondsRangeRule {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBudgetBurnMetricRule for RollingBudgetBurnMetricWindowDaysRangeRule {
    fn check(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling budget burn metric.
#[derive(Debug, Clone)]
pub struct RollingBudgetBurnMetricValidator {
    pub rules: Vec<Rc<dyn RollingBudgetBurnMetricRule>>,
}

impl RollingBudgetBurnMetricValidator {
    pub fn validate(&self, item: &RollingBudgetBurnMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingBudgetBurnMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingBudgetBurnMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_budget_burn_metric_validator() -> RollingBudgetBurnMetricValidator {
    RollingBudgetBurnMetricValidator {
        rules: vec![
            Rc::new(RollingBudgetBurnMetricTitleLengthRule { max_length: 32 }) as Rc<dyn RollingBudgetBurnMetricRule>,
            Rc::new(RollingBudgetBurnMetricRequiredTitleRule {
                message: "rolling budget burn metric needs this field".to_string(),
            }),
            Rc::new(RollingBudgetBurnMetricCodeLengthRule { max_length: 32 }),
            Rc::new(RollingBudgetBurnMetricRequiredCodeRule {
                message: "rolling budget burn metric needs this field".to_string(),
            }),
            Rc::new(RollingBudgetBurnMetricTtlSecondsRangeRule { min: 255, max: 1000 }),
            Rc::new(RollingBudgetBurnMetricWindowDaysRangeRule { min: 16, max: 255 }),
        ],
    }
}
