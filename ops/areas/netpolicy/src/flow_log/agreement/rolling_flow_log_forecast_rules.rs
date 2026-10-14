use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling flow log forecast must pass. Check returns its problems, or none.
pub trait RollingFlowLogForecastRule: std::fmt::Debug {
    fn check(&self, item: &RollingFlowLogForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogForecastPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingFlowLogForecastRule for RollingFlowLogForecastPriorityRangeRule {
    fn check(&self, item: &RollingFlowLogForecast) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogForecastRequiredHostRule {
    pub message: String,
}

impl RollingFlowLogForecastRule for RollingFlowLogForecastRequiredHostRule {
    fn check(&self, item: &RollingFlowLogForecast) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogForecastTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingFlowLogForecastRule for RollingFlowLogForecastTtlSecondsRangeRule {
    fn check(&self, item: &RollingFlowLogForecast) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingFlowLogForecastRequiredClusterRule {
    pub message: String,
}

impl RollingFlowLogForecastRule for RollingFlowLogForecastRequiredClusterRule {
    fn check(&self, item: &RollingFlowLogForecast) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling flow log forecast.
#[derive(Debug, Clone)]
pub struct RollingFlowLogForecastValidator {
    pub rules: Vec<Rc<dyn RollingFlowLogForecastRule>>,
}

impl RollingFlowLogForecastValidator {
    pub fn validate(&self, item: &RollingFlowLogForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingFlowLogForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingFlowLogForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_flow_log_forecast_validator() -> RollingFlowLogForecastValidator {
    RollingFlowLogForecastValidator {
        rules: vec![
            Rc::new(RollingFlowLogForecastPriorityRangeRule { min: 16, max: 3600 })
                as Rc<dyn RollingFlowLogForecastRule>,
            Rc::new(RollingFlowLogForecastRequiredHostRule {
                message: "rolling flow log forecast needs an entry".to_string(),
            }),
            Rc::new(RollingFlowLogForecastTtlSecondsRangeRule { min: 120, max: 3600 }),
            Rc::new(RollingFlowLogForecastRequiredClusterRule {
                message: "rolling flow log forecast needs this field".to_string(),
            }),
        ],
    }
}
