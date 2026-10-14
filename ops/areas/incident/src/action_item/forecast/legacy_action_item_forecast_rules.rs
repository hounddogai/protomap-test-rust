use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy action item forecast must pass. Check returns its problems, or none.
pub trait LegacyActionItemForecastRule: std::fmt::Debug {
    fn check(&self, item: &LegacyActionItemForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemForecastCodeLengthRule {
    pub max_length: i64,
}

impl LegacyActionItemForecastRule for LegacyActionItemForecastCodeLengthRule {
    fn check(&self, item: &LegacyActionItemForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemForecastMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyActionItemForecastRule for LegacyActionItemForecastMinimumRangeRule {
    fn check(&self, item: &LegacyActionItemForecast) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemForecastApprovedGuardRule {
    pub limit: i64,
}

impl LegacyActionItemForecastRule for LegacyActionItemForecastApprovedGuardRule {
    fn check(&self, item: &LegacyActionItemForecast) -> Vec<String> {
        if item.approved && (item.attempts > self.limit) {
            return vec![format!(
                "attempts {} exceeds {} while approved",
                item.attempts, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemForecastRequiredNameRule {
    pub message: String,
}

impl LegacyActionItemForecastRule for LegacyActionItemForecastRequiredNameRule {
    fn check(&self, item: &LegacyActionItemForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy action item forecast.
#[derive(Debug, Clone)]
pub struct LegacyActionItemForecastValidator {
    pub rules: Vec<Rc<dyn LegacyActionItemForecastRule>>,
}

impl LegacyActionItemForecastValidator {
    pub fn validate(&self, item: &LegacyActionItemForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyActionItemForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyActionItemForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_action_item_forecast_validator() -> LegacyActionItemForecastValidator {
    LegacyActionItemForecastValidator {
        rules: vec![
            Rc::new(LegacyActionItemForecastCodeLengthRule { max_length: 64 }) as Rc<dyn LegacyActionItemForecastRule>,
            Rc::new(LegacyActionItemForecastMinimumRangeRule { min: 4.3, max: 4987.6 }),
            Rc::new(LegacyActionItemForecastApprovedGuardRule { limit: 64 }),
            Rc::new(LegacyActionItemForecastRequiredNameRule {
                message: "legacy action item forecast needs this field".to_string(),
            }),
        ],
    }
}
