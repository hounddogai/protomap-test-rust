use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard batch must pass. Check returns its problems, or none.
pub trait DashboardBatchRule: std::fmt::Debug {
    fn check(&self, item: &DashboardBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchRequiredIssuerRule {
    pub message: String,
}

impl DashboardBatchRule for DashboardBatchRequiredIssuerRule {
    fn check(&self, item: &DashboardBatch) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchNameLengthRule {
    pub max_length: i64,
}

impl DashboardBatchRule for DashboardBatchNameLengthRule {
    fn check(&self, item: &DashboardBatch) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardBatchRule for DashboardBatchPriorityRangeRule {
    fn check(&self, item: &DashboardBatch) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchPhoneLengthRule {
    pub max_length: i64,
}

impl DashboardBatchRule for DashboardBatchPhoneLengthRule {
    fn check(&self, item: &DashboardBatch) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardBatchIssuerLengthRule {
    pub max_length: i64,
}

impl DashboardBatchRule for DashboardBatchIssuerLengthRule {
    fn check(&self, item: &DashboardBatch) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard batch.
#[derive(Debug, Clone)]
pub struct DashboardBatchValidator {
    pub rules: Vec<Rc<dyn DashboardBatchRule>>,
}

impl DashboardBatchValidator {
    pub fn validate(&self, item: &DashboardBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_batch_validator() -> DashboardBatchValidator {
    DashboardBatchValidator {
        rules: vec![
            Rc::new(DashboardBatchRequiredIssuerRule {
                message: "dashboard batch needs this field".to_string(),
            }) as Rc<dyn DashboardBatchRule>,
            Rc::new(DashboardBatchNameLengthRule { max_length: 32 }),
            Rc::new(DashboardBatchPriorityRangeRule { min: 16, max: 86400 }),
            Rc::new(DashboardBatchPhoneLengthRule { max_length: 255 }),
            Rc::new(DashboardBatchIssuerLengthRule { max_length: 1000 }),
        ],
    }
}
