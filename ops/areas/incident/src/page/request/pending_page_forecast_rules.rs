use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending page forecast must pass. Check returns its problems, or none.
pub trait PendingPageForecastRule: std::fmt::Debug {
    fn check(&self, item: &PendingPageForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastCodePrefixRule {
    pub prefix: String,
}

impl PendingPageForecastRule for PendingPageForecastCodePrefixRule {
    fn check(&self, item: &PendingPageForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastRequiredSegmentRule {
    pub message: String,
}

impl PendingPageForecastRule for PendingPageForecastRequiredSegmentRule {
    fn check(&self, item: &PendingPageForecast) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingPageForecastRule for PendingPageForecastRatioRangeRule {
    fn check(&self, item: &PendingPageForecast) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastCodeLengthRule {
    pub max_length: i64,
}

impl PendingPageForecastRule for PendingPageForecastCodeLengthRule {
    fn check(&self, item: &PendingPageForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPageForecastRequiredOwnerIdRule {
    pub message: String,
}

impl PendingPageForecastRule for PendingPageForecastRequiredOwnerIdRule {
    fn check(&self, item: &PendingPageForecast) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending page forecast.
#[derive(Debug, Clone)]
pub struct PendingPageForecastValidator {
    pub rules: Vec<Rc<dyn PendingPageForecastRule>>,
}

impl PendingPageForecastValidator {
    pub fn validate(&self, item: &PendingPageForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPageForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPageForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_page_forecast_validator() -> PendingPageForecastValidator {
    PendingPageForecastValidator {
        rules: vec![
            Rc::new(PendingPageForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn PendingPageForecastRule>,
            Rc::new(PendingPageForecastRequiredSegmentRule {
                message: "pending page forecast needs an entry".to_string(),
            }),
            Rc::new(PendingPageForecastRatioRangeRule { min: 4.5, max: 3187.3 }),
            Rc::new(PendingPageForecastCodeLengthRule { max_length: 86400 }),
            Rc::new(PendingPageForecastRequiredOwnerIdRule {
                message: "pending page forecast needs an entry".to_string(),
            }),
        ],
    }
}
