use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment forecast must pass. Check returns its problems, or none.
pub trait SegmentForecastRule: std::fmt::Debug {
    fn check(&self, item: &SegmentForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastRequiredLabelRule {
    pub message: String,
}

impl SegmentForecastRule for SegmentForecastRequiredLabelRule {
    fn check(&self, item: &SegmentForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastKeyIdLengthRule {
    pub max_length: i64,
}

impl SegmentForecastRule for SegmentForecastKeyIdLengthRule {
    fn check(&self, item: &SegmentForecast) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl SegmentForecastRule for SegmentForecastAllowedLabelRule {
    fn check(&self, item: &SegmentForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentForecastRule for SegmentForecastBurstRangeRule {
    fn check(&self, item: &SegmentForecast) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastRequiredCodeRule {
    pub message: String,
}

impl SegmentForecastRule for SegmentForecastRequiredCodeRule {
    fn check(&self, item: &SegmentForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentForecastAudienceLengthRule {
    pub max_length: i64,
}

impl SegmentForecastRule for SegmentForecastAudienceLengthRule {
    fn check(&self, item: &SegmentForecast) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment forecast.
#[derive(Debug, Clone)]
pub struct SegmentForecastValidator {
    pub rules: Vec<Rc<dyn SegmentForecastRule>>,
}

impl SegmentForecastValidator {
    pub fn validate(&self, item: &SegmentForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SegmentForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_forecast_validator() -> SegmentForecastValidator {
    SegmentForecastValidator {
        rules: vec![
            Rc::new(SegmentForecastRequiredLabelRule {
                message: "segment forecast needs a value".to_string(),
            }) as Rc<dyn SegmentForecastRule>,
            Rc::new(SegmentForecastKeyIdLengthRule { max_length: 16 }),
            Rc::new(SegmentForecastAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(SegmentForecastBurstRangeRule { min: 64, max: 1000 }),
            Rc::new(SegmentForecastRequiredCodeRule {
                message: "segment forecast needs this field".to_string(),
            }),
            Rc::new(SegmentForecastAudienceLengthRule { max_length: 80 }),
        ],
    }
}
