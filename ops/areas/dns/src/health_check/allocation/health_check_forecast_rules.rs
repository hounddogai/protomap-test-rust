use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check forecast must pass. Check returns its problems, or none.
pub trait HealthCheckForecastRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckForecastRequiredLabelRule {
    pub message: String,
}

impl HealthCheckForecastRule for HealthCheckForecastRequiredLabelRule {
    fn check(&self, item: &HealthCheckForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckForecastRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckForecastRule for HealthCheckForecastRotationDaysRangeRule {
    fn check(&self, item: &HealthCheckForecast) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckForecastLabelLengthRule {
    pub max_length: i64,
}

impl HealthCheckForecastRule for HealthCheckForecastLabelLengthRule {
    fn check(&self, item: &HealthCheckForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckForecastCodeLengthRule {
    pub max_length: i64,
}

impl HealthCheckForecastRule for HealthCheckForecastCodeLengthRule {
    fn check(&self, item: &HealthCheckForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckForecastWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckForecastRule for HealthCheckForecastWeightRangeRule {
    fn check(&self, item: &HealthCheckForecast) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a health check forecast.
#[derive(Debug, Clone)]
pub struct HealthCheckForecastValidator {
    pub rules: Vec<Rc<dyn HealthCheckForecastRule>>,
}

impl HealthCheckForecastValidator {
    pub fn validate(&self, item: &HealthCheckForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_forecast_validator() -> HealthCheckForecastValidator {
    HealthCheckForecastValidator {
        rules: vec![
            Rc::new(HealthCheckForecastRequiredLabelRule {
                message: "health check forecast needs a value".to_string(),
            }) as Rc<dyn HealthCheckForecastRule>,
            Rc::new(HealthCheckForecastRotationDaysRangeRule { min: 120, max: 3600 }),
            Rc::new(HealthCheckForecastLabelLengthRule { max_length: 16 }),
            Rc::new(HealthCheckForecastCodeLengthRule { max_length: 16 }),
            Rc::new(HealthCheckForecastWeightRangeRule { min: 2.5, max: 160.6 }),
        ],
    }
}
