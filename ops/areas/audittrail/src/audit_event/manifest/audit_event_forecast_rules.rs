use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event forecast must pass. Check returns its problems, or none.
pub trait AuditEventForecastRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecastCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventForecastRule for AuditEventForecastCapacityRangeRule {
    fn check(&self, item: &AuditEventForecast) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecastRequiredLabelRule {
    pub message: String,
}

impl AuditEventForecastRule for AuditEventForecastRequiredLabelRule {
    fn check(&self, item: &AuditEventForecast) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecastRequiredCodeRule {
    pub message: String,
}

impl AuditEventForecastRule for AuditEventForecastRequiredCodeRule {
    fn check(&self, item: &AuditEventForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecastCurrencyLengthRule {
    pub max_length: i64,
}

impl AuditEventForecastRule for AuditEventForecastCurrencyLengthRule {
    fn check(&self, item: &AuditEventForecast) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecastCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventForecastRule for AuditEventForecastCodeLengthRule {
    fn check(&self, item: &AuditEventForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventForecastRequiredCurrencyRule {
    pub message: String,
}

impl AuditEventForecastRule for AuditEventForecastRequiredCurrencyRule {
    fn check(&self, item: &AuditEventForecast) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event forecast.
#[derive(Debug, Clone)]
pub struct AuditEventForecastValidator {
    pub rules: Vec<Rc<dyn AuditEventForecastRule>>,
}

impl AuditEventForecastValidator {
    pub fn validate(&self, item: &AuditEventForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_forecast_validator() -> AuditEventForecastValidator {
    AuditEventForecastValidator {
        rules: vec![
            Rc::new(AuditEventForecastCapacityRangeRule { min: 3600, max: 3600 }) as Rc<dyn AuditEventForecastRule>,
            Rc::new(AuditEventForecastRequiredLabelRule {
                message: "audit event forecast needs an entry".to_string(),
            }),
            Rc::new(AuditEventForecastRequiredCodeRule {
                message: "audit event forecast needs an entry".to_string(),
            }),
            Rc::new(AuditEventForecastCurrencyLengthRule { max_length: 255 }),
            Rc::new(AuditEventForecastCodeLengthRule { max_length: 1000 }),
            Rc::new(AuditEventForecastRequiredCurrencyRule {
                message: "audit event forecast needs a value".to_string(),
            }),
        ],
    }
}
