use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default forecast segment must pass. Check returns its problems, or none.
pub trait DefaultForecastSegmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultForecastSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegmentChecksumLengthRule {
    pub max_length: i64,
}

impl DefaultForecastSegmentRule for DefaultForecastSegmentChecksumLengthRule {
    fn check(&self, item: &DefaultForecastSegment) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegmentNamespaceLengthRule {
    pub max_length: i64,
}

impl DefaultForecastSegmentRule for DefaultForecastSegmentNamespaceLengthRule {
    fn check(&self, item: &DefaultForecastSegment) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegmentRequiredNamespaceRule {
    pub message: String,
}

impl DefaultForecastSegmentRule for DefaultForecastSegmentRequiredNamespaceRule {
    fn check(&self, item: &DefaultForecastSegment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultForecastSegmentRule for DefaultForecastSegmentCreatedAtRangeRule {
    fn check(&self, item: &DefaultForecastSegment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultForecastSegmentRule for DefaultForecastSegmentCodeLengthRule {
    fn check(&self, item: &DefaultForecastSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default forecast segment.
#[derive(Debug, Clone)]
pub struct DefaultForecastSegmentValidator {
    pub rules: Vec<Rc<dyn DefaultForecastSegmentRule>>,
}

impl DefaultForecastSegmentValidator {
    pub fn validate(&self, item: &DefaultForecastSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultForecastSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultForecastSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_forecast_segment_validator() -> DefaultForecastSegmentValidator {
    DefaultForecastSegmentValidator {
        rules: vec![
            Rc::new(DefaultForecastSegmentChecksumLengthRule { max_length: 86400 })
                as Rc<dyn DefaultForecastSegmentRule>,
            Rc::new(DefaultForecastSegmentNamespaceLengthRule { max_length: 32 }),
            Rc::new(DefaultForecastSegmentRequiredNamespaceRule {
                message: "default forecast segment needs this field".to_string(),
            }),
            Rc::new(DefaultForecastSegmentCreatedAtRangeRule { min: 16, max: 32 }),
            Rc::new(DefaultForecastSegmentCodeLengthRule { max_length: 80 }),
        ],
    }
}
