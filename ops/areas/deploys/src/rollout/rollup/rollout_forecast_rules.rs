use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout forecast must pass. Check returns its problems, or none.
pub trait RolloutForecastRule: std::fmt::Debug {
    fn check(&self, item: &RolloutForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutForecastRequiredCodeRule {
    pub message: String,
}

impl RolloutForecastRule for RolloutForecastRequiredCodeRule {
    fn check(&self, item: &RolloutForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutForecastTitleLengthRule {
    pub max_length: i64,
}

impl RolloutForecastRule for RolloutForecastTitleLengthRule {
    fn check(&self, item: &RolloutForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutForecastRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutForecastRule for RolloutForecastRatioRangeRule {
    fn check(&self, item: &RolloutForecast) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rollout forecast.
#[derive(Debug, Clone)]
pub struct RolloutForecastValidator {
    pub rules: Vec<Rc<dyn RolloutForecastRule>>,
}

impl RolloutForecastValidator {
    pub fn validate(&self, item: &RolloutForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_forecast_validator() -> RolloutForecastValidator {
    RolloutForecastValidator {
        rules: vec![
            Rc::new(RolloutForecastRequiredCodeRule {
                message: "rollout forecast needs this field".to_string(),
            }) as Rc<dyn RolloutForecastRule>,
            Rc::new(RolloutForecastTitleLengthRule { max_length: 3600 }),
            Rc::new(RolloutForecastRatioRangeRule { min: 3.1, max: 3834.2 }),
        ],
    }
}
