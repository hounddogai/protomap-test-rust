use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted segment forecast must pass. Check returns its problems, or none.
pub trait WeightedSegmentForecastRule: std::fmt::Debug {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedSegmentForecastRule for WeightedSegmentForecastSequenceRangeRule {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastTitleLengthRule {
    pub max_length: i64,
}

impl WeightedSegmentForecastRule for WeightedSegmentForecastTitleLengthRule {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl WeightedSegmentForecastRule for WeightedSegmentForecastTagsLimitRule {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedSegmentForecastRule for WeightedSegmentForecastRotationDaysRangeRule {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String> {
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
pub struct WeightedSegmentForecastRequiredCodeRule {
    pub message: String,
}

impl WeightedSegmentForecastRule for WeightedSegmentForecastRequiredCodeRule {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentForecastRequiredTimezoneRule {
    pub message: String,
}

impl WeightedSegmentForecastRule for WeightedSegmentForecastRequiredTimezoneRule {
    fn check(&self, item: &WeightedSegmentForecast) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted segment forecast.
#[derive(Debug, Clone)]
pub struct WeightedSegmentForecastValidator {
    pub rules: Vec<Rc<dyn WeightedSegmentForecastRule>>,
}

impl WeightedSegmentForecastValidator {
    pub fn validate(&self, item: &WeightedSegmentForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedSegmentForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedSegmentForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_segment_forecast_validator() -> WeightedSegmentForecastValidator {
    WeightedSegmentForecastValidator {
        rules: vec![
            Rc::new(WeightedSegmentForecastSequenceRangeRule { min: 16, max: 255 })
                as Rc<dyn WeightedSegmentForecastRule>,
            Rc::new(WeightedSegmentForecastTitleLengthRule { max_length: 16 }),
            Rc::new(WeightedSegmentForecastTagsLimitRule {
                max_count: 1000,
                banned: vec!["weighted".to_string()],
            }),
            Rc::new(WeightedSegmentForecastRotationDaysRangeRule { min: 16, max: 1000 }),
            Rc::new(WeightedSegmentForecastRequiredCodeRule {
                message: "weighted segment forecast needs this field".to_string(),
            }),
            Rc::new(WeightedSegmentForecastRequiredTimezoneRule {
                message: "weighted segment forecast needs a value".to_string(),
            }),
        ],
    }
}
