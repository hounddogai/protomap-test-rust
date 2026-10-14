use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering forecast must pass. Check returns its problems, or none.
pub trait PeeringForecastRule: std::fmt::Debug {
    fn check(&self, item: &PeeringForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastApprovedGuardRule {
    pub limit: i64,
}

impl PeeringForecastRule for PeeringForecastApprovedGuardRule {
    fn check(&self, item: &PeeringForecast) -> Vec<String> {
        if item.approved && (item.ends_at > self.limit) {
            return vec![format!(
                "ends at {} exceeds {} while approved",
                item.ends_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastTitleLengthRule {
    pub max_length: i64,
}

impl PeeringForecastRule for PeeringForecastTitleLengthRule {
    fn check(&self, item: &PeeringForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastRequiredTimezoneRule {
    pub message: String,
}

impl PeeringForecastRule for PeeringForecastRequiredTimezoneRule {
    fn check(&self, item: &PeeringForecast) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastCodePrefixRule {
    pub prefix: String,
}

impl PeeringForecastRule for PeeringForecastCodePrefixRule {
    fn check(&self, item: &PeeringForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringForecastRule for PeeringForecastEndsAtRangeRule {
    fn check(&self, item: &PeeringForecast) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringForecastCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringForecastRule for PeeringForecastCapacityRangeRule {
    fn check(&self, item: &PeeringForecast) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering forecast.
#[derive(Debug, Clone)]
pub struct PeeringForecastValidator {
    pub rules: Vec<Rc<dyn PeeringForecastRule>>,
}

impl PeeringForecastValidator {
    pub fn validate(&self, item: &PeeringForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_forecast_validator() -> PeeringForecastValidator {
    PeeringForecastValidator {
        rules: vec![
            Rc::new(PeeringForecastApprovedGuardRule { limit: 120 }) as Rc<dyn PeeringForecastRule>,
            Rc::new(PeeringForecastTitleLengthRule { max_length: 255 }),
            Rc::new(PeeringForecastRequiredTimezoneRule {
                message: "peering forecast needs a value".to_string(),
            }),
            Rc::new(PeeringForecastCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(PeeringForecastEndsAtRangeRule { min: 80, max: 86400 }),
            Rc::new(PeeringForecastCapacityRangeRule { min: 64, max: 1000 }),
        ],
    }
}
