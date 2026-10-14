use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast item must pass. Check returns its problems, or none.
pub trait ForecastItemRule: std::fmt::Debug {
    fn check(&self, item: &ForecastItem) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastItemRule for ForecastItemRateRangeRule {
    fn check(&self, item: &ForecastItem) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemArchivedGuardRule {
    pub limit: f64,
}

impl ForecastItemRule for ForecastItemArchivedGuardRule {
    fn check(&self, item: &ForecastItem) -> Vec<String> {
        if item.archived && (item.rate > self.limit) {
            return vec![format!("rate {} exceeds {} while archived", item.rate, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemRequiredCodeRule {
    pub message: String,
}

impl ForecastItemRule for ForecastItemRequiredCodeRule {
    fn check(&self, item: &ForecastItem) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ForecastItemRule for ForecastItemLimitRangeRule {
    fn check(&self, item: &ForecastItem) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastItemLabelLengthRule {
    pub max_length: i64,
}

impl ForecastItemRule for ForecastItemLabelLengthRule {
    fn check(&self, item: &ForecastItem) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast item.
#[derive(Debug, Clone)]
pub struct ForecastItemValidator {
    pub rules: Vec<Rc<dyn ForecastItemRule>>,
}

impl ForecastItemValidator {
    pub fn validate(&self, item: &ForecastItem) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ForecastItem) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastItem]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_item_validator() -> ForecastItemValidator {
    ForecastItemValidator {
        rules: vec![
            Rc::new(ForecastItemRateRangeRule { min: 10.0, max: 884.8 }) as Rc<dyn ForecastItemRule>,
            Rc::new(ForecastItemArchivedGuardRule { limit: 9.5 }),
            Rc::new(ForecastItemRequiredCodeRule {
                message: "forecast item needs this field".to_string(),
            }),
            Rc::new(ForecastItemLimitRangeRule { min: 16, max: 80 }),
            Rc::new(ForecastItemLabelLengthRule { max_length: 64 }),
        ],
    }
}
