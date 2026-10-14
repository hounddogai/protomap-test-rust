use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual allowlist forecast must pass. Check returns its problems, or none.
pub trait ManualAllowlistForecastRule: std::fmt::Debug {
    fn check(&self, item: &ManualAllowlistForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistForecastRequiredUnitRule {
    pub message: String,
}

impl ManualAllowlistForecastRule for ManualAllowlistForecastRequiredUnitRule {
    fn check(&self, item: &ManualAllowlistForecast) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistForecastLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualAllowlistForecastRule for ManualAllowlistForecastLimitRangeRule {
    fn check(&self, item: &ManualAllowlistForecast) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistForecastRequiredNamespaceRule {
    pub message: String,
}

impl ManualAllowlistForecastRule for ManualAllowlistForecastRequiredNamespaceRule {
    fn check(&self, item: &ManualAllowlistForecast) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual allowlist forecast.
#[derive(Debug, Clone)]
pub struct ManualAllowlistForecastValidator {
    pub rules: Vec<Rc<dyn ManualAllowlistForecastRule>>,
}

impl ManualAllowlistForecastValidator {
    pub fn validate(&self, item: &ManualAllowlistForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualAllowlistForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAllowlistForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_allowlist_forecast_validator() -> ManualAllowlistForecastValidator {
    ManualAllowlistForecastValidator {
        rules: vec![
            Rc::new(ManualAllowlistForecastRequiredUnitRule {
                message: "manual allowlist forecast needs a value".to_string(),
            }) as Rc<dyn ManualAllowlistForecastRule>,
            Rc::new(ManualAllowlistForecastLimitRangeRule { min: 32, max: 3600 }),
            Rc::new(ManualAllowlistForecastRequiredNamespaceRule {
                message: "manual allowlist forecast needs a value".to_string(),
            }),
        ],
    }
}
