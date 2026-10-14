use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a region pin forecast must pass. Check returns its problems, or none.
pub trait RegionPinForecastRule: std::fmt::Debug {
    fn check(&self, item: &RegionPinForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinForecastRequiredNameRule {
    pub message: String,
}

impl RegionPinForecastRule for RegionPinForecastRequiredNameRule {
    fn check(&self, item: &RegionPinForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinForecastRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionPinForecastRule for RegionPinForecastRatioRangeRule {
    fn check(&self, item: &RegionPinForecast) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinForecastCodePrefixRule {
    pub prefix: String,
}

impl RegionPinForecastRule for RegionPinForecastCodePrefixRule {
    fn check(&self, item: &RegionPinForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinForecastRequiredCodeRule {
    pub message: String,
}

impl RegionPinForecastRule for RegionPinForecastRequiredCodeRule {
    fn check(&self, item: &RegionPinForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a region pin forecast.
#[derive(Debug, Clone)]
pub struct RegionPinForecastValidator {
    pub rules: Vec<Rc<dyn RegionPinForecastRule>>,
}

impl RegionPinForecastValidator {
    pub fn validate(&self, item: &RegionPinForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionPinForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionPinForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_region_pin_forecast_validator() -> RegionPinForecastValidator {
    RegionPinForecastValidator {
        rules: vec![
            Rc::new(RegionPinForecastRequiredNameRule {
                message: "region pin forecast needs this field".to_string(),
            }) as Rc<dyn RegionPinForecastRule>,
            Rc::new(RegionPinForecastRatioRangeRule { min: 9.0, max: 3765.3 }),
            Rc::new(RegionPinForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RegionPinForecastRequiredCodeRule {
                message: "region pin forecast needs an entry".to_string(),
            }),
        ],
    }
}
