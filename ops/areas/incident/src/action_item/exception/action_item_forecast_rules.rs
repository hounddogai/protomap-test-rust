use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a action item forecast must pass. Check returns its problems, or none.
pub trait ActionItemForecastRule: std::fmt::Debug {
    fn check(&self, item: &ActionItemForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecastRequiredLabelRule {
    pub message: String,
}

impl ActionItemForecastRule for ActionItemForecastRequiredLabelRule {
    fn check(&self, item: &ActionItemForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecastCodeLengthRule {
    pub max_length: i64,
}

impl ActionItemForecastRule for ActionItemForecastCodeLengthRule {
    fn check(&self, item: &ActionItemForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecastOwnerIdLengthRule {
    pub max_length: i64,
}

impl ActionItemForecastRule for ActionItemForecastOwnerIdLengthRule {
    fn check(&self, item: &ActionItemForecast) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecastThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ActionItemForecastRule for ActionItemForecastThresholdRangeRule {
    fn check(&self, item: &ActionItemForecast) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemForecastCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ActionItemForecastRule for ActionItemForecastCapacityRangeRule {
    fn check(&self, item: &ActionItemForecast) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a action item forecast.
#[derive(Debug, Clone)]
pub struct ActionItemForecastValidator {
    pub rules: Vec<Rc<dyn ActionItemForecastRule>>,
}

impl ActionItemForecastValidator {
    pub fn validate(&self, item: &ActionItemForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ActionItemForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ActionItemForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_action_item_forecast_validator() -> ActionItemForecastValidator {
    ActionItemForecastValidator {
        rules: vec![
            Rc::new(ActionItemForecastRequiredLabelRule {
                message: "action item forecast needs a value".to_string(),
            }) as Rc<dyn ActionItemForecastRule>,
            Rc::new(ActionItemForecastCodeLengthRule { max_length: 120 }),
            Rc::new(ActionItemForecastOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(ActionItemForecastThresholdRangeRule { min: 6.8, max: 893.3 }),
            Rc::new(ActionItemForecastCapacityRangeRule { min: 16, max: 120 }),
        ],
    }
}
