use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling dashboard summary must pass. Check returns its problems, or none.
pub trait RollingDashboardSummaryRule: std::fmt::Debug {
    fn check(&self, item: &RollingDashboardSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingDashboardSummaryRule for RollingDashboardSummaryCostRangeRule {
    fn check(&self, item: &RollingDashboardSummary) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryCodePrefixRule {
    pub prefix: String,
}

impl RollingDashboardSummaryRule for RollingDashboardSummaryCodePrefixRule {
    fn check(&self, item: &RollingDashboardSummary) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryRequiredTimezoneRule {
    pub message: String,
}

impl RollingDashboardSummaryRule for RollingDashboardSummaryRequiredTimezoneRule {
    fn check(&self, item: &RollingDashboardSummary) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryRequiredKeyIdRule {
    pub message: String,
}

impl RollingDashboardSummaryRule for RollingDashboardSummaryRequiredKeyIdRule {
    fn check(&self, item: &RollingDashboardSummary) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryCodeLengthRule {
    pub max_length: i64,
}

impl RollingDashboardSummaryRule for RollingDashboardSummaryCodeLengthRule {
    fn check(&self, item: &RollingDashboardSummary) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling dashboard summary.
#[derive(Debug, Clone)]
pub struct RollingDashboardSummaryValidator {
    pub rules: Vec<Rc<dyn RollingDashboardSummaryRule>>,
}

impl RollingDashboardSummaryValidator {
    pub fn validate(&self, item: &RollingDashboardSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingDashboardSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingDashboardSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_dashboard_summary_validator() -> RollingDashboardSummaryValidator {
    RollingDashboardSummaryValidator {
        rules: vec![
            Rc::new(RollingDashboardSummaryCostRangeRule { min: 9.9, max: 4264.3 })
                as Rc<dyn RollingDashboardSummaryRule>,
            Rc::new(RollingDashboardSummaryCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RollingDashboardSummaryRequiredTimezoneRule {
                message: "rolling dashboard summary needs an entry".to_string(),
            }),
            Rc::new(RollingDashboardSummaryRequiredKeyIdRule {
                message: "rolling dashboard summary needs this field".to_string(),
            }),
            Rc::new(RollingDashboardSummaryCodeLengthRule { max_length: 80 }),
        ],
    }
}
