use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollout metric must pass. Check returns its problems, or none.
pub trait LegacyRolloutMetricRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRolloutMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetricFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRolloutMetricRule for LegacyRolloutMetricFloorValueRangeRule {
    fn check(&self, item: &LegacyRolloutMetric) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetricRequiredCodeRule {
    pub message: String,
}

impl LegacyRolloutMetricRule for LegacyRolloutMetricRequiredCodeRule {
    fn check(&self, item: &LegacyRolloutMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetricCodePrefixRule {
    pub prefix: String,
}

impl LegacyRolloutMetricRule for LegacyRolloutMetricCodePrefixRule {
    fn check(&self, item: &LegacyRolloutMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetricBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyRolloutMetricRule for LegacyRolloutMetricBudgetRangeRule {
    fn check(&self, item: &LegacyRolloutMetric) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollout metric.
#[derive(Debug, Clone)]
pub struct LegacyRolloutMetricValidator {
    pub rules: Vec<Rc<dyn LegacyRolloutMetricRule>>,
}

impl LegacyRolloutMetricValidator {
    pub fn validate(&self, item: &LegacyRolloutMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyRolloutMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRolloutMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollout_metric_validator() -> LegacyRolloutMetricValidator {
    LegacyRolloutMetricValidator {
        rules: vec![
            Rc::new(LegacyRolloutMetricFloorValueRangeRule { min: 5.3, max: 2543.2 })
                as Rc<dyn LegacyRolloutMetricRule>,
            Rc::new(LegacyRolloutMetricRequiredCodeRule {
                message: "legacy rollout metric needs an entry".to_string(),
            }),
            Rc::new(LegacyRolloutMetricCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(LegacyRolloutMetricBudgetRangeRule { min: 7.9, max: 2567.7 }),
        ],
    }
}
