use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched measurement request must pass. Check returns its problems, or none.
pub trait BatchedMeasurementRequestRule: std::fmt::Debug {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequestNameLengthRule {
    pub max_length: i64,
}

impl BatchedMeasurementRequestRule for BatchedMeasurementRequestNameLengthRule {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequestGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedMeasurementRequestRule for BatchedMeasurementRequestGraceDaysRangeRule {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequestRequiredNameRule {
    pub message: String,
}

impl BatchedMeasurementRequestRule for BatchedMeasurementRequestRequiredNameRule {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequestOwnerIdLengthRule {
    pub max_length: i64,
}

impl BatchedMeasurementRequestRule for BatchedMeasurementRequestOwnerIdLengthRule {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
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
pub struct BatchedMeasurementRequestTitleLengthRule {
    pub max_length: i64,
}

impl BatchedMeasurementRequestRule for BatchedMeasurementRequestTitleLengthRule {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMeasurementRequestAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl BatchedMeasurementRequestRule for BatchedMeasurementRequestAllowedOwnerIdRule {
    fn check(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is blocked", item.owner_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched measurement request.
#[derive(Debug, Clone)]
pub struct BatchedMeasurementRequestValidator {
    pub rules: Vec<Rc<dyn BatchedMeasurementRequestRule>>,
}

impl BatchedMeasurementRequestValidator {
    pub fn validate(&self, item: &BatchedMeasurementRequest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedMeasurementRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedMeasurementRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_measurement_request_validator() -> BatchedMeasurementRequestValidator {
    BatchedMeasurementRequestValidator {
        rules: vec![
            Rc::new(BatchedMeasurementRequestNameLengthRule { max_length: 255 })
                as Rc<dyn BatchedMeasurementRequestRule>,
            Rc::new(BatchedMeasurementRequestGraceDaysRangeRule { min: 1000, max: 1000 }),
            Rc::new(BatchedMeasurementRequestRequiredNameRule {
                message: "batched measurement request needs a value".to_string(),
            }),
            Rc::new(BatchedMeasurementRequestOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(BatchedMeasurementRequestTitleLengthRule { max_length: 3600 }),
            Rc::new(BatchedMeasurementRequestAllowedOwnerIdRule {
                allowed: vec!["seasonal".to_string()],
            }),
        ],
    }
}
