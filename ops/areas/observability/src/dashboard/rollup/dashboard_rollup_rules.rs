use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard rollup must pass. Check returns its problems, or none.
pub trait DashboardRollupRule: std::fmt::Debug {
    fn check(&self, item: &DashboardRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardRollupRule for DashboardRollupEndsAtRangeRule {
    fn check(&self, item: &DashboardRollup) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardRollupRule for DashboardRollupExpiresAtRangeRule {
    fn check(&self, item: &DashboardRollup) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupRequiredLabelRule {
    pub message: String,
}

impl DashboardRollupRule for DashboardRollupRequiredLabelRule {
    fn check(&self, item: &DashboardRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupRequiredCurrencyRule {
    pub message: String,
}

impl DashboardRollupRule for DashboardRollupRequiredCurrencyRule {
    fn check(&self, item: &DashboardRollup) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupCurrencyLengthRule {
    pub max_length: i64,
}

impl DashboardRollupRule for DashboardRollupCurrencyLengthRule {
    fn check(&self, item: &DashboardRollup) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardRollupRule for DashboardRollupTtlSecondsRangeRule {
    fn check(&self, item: &DashboardRollup) -> Vec<String> {
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

/// Runs every rule of a dashboard rollup.
#[derive(Debug, Clone)]
pub struct DashboardRollupValidator {
    pub rules: Vec<Rc<dyn DashboardRollupRule>>,
}

impl DashboardRollupValidator {
    pub fn validate(&self, item: &DashboardRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_rollup_validator() -> DashboardRollupValidator {
    DashboardRollupValidator {
        rules: vec![
            Rc::new(DashboardRollupEndsAtRangeRule { min: 3600, max: 86400 }) as Rc<dyn DashboardRollupRule>,
            Rc::new(DashboardRollupExpiresAtRangeRule { min: 255, max: 86400 }),
            Rc::new(DashboardRollupRequiredLabelRule {
                message: "dashboard rollup needs this field".to_string(),
            }),
            Rc::new(DashboardRollupRequiredCurrencyRule {
                message: "dashboard rollup needs this field".to_string(),
            }),
            Rc::new(DashboardRollupCurrencyLengthRule { max_length: 120 }),
            Rc::new(DashboardRollupTtlSecondsRangeRule { min: 120, max: 255 }),
        ],
    }
}
