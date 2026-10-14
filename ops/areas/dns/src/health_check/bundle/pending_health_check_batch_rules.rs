use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending health check batch must pass. Check returns its problems, or none.
pub trait PendingHealthCheckBatchRule: std::fmt::Debug {
    fn check(&self, item: &PendingHealthCheckBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckBatchKeyIdLengthRule {
    pub max_length: i64,
}

impl PendingHealthCheckBatchRule for PendingHealthCheckBatchKeyIdLengthRule {
    fn check(&self, item: &PendingHealthCheckBatch) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckBatchRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingHealthCheckBatchRule for PendingHealthCheckBatchRevisionRangeRule {
    fn check(&self, item: &PendingHealthCheckBatch) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckBatchRequiredKeyIdRule {
    pub message: String,
}

impl PendingHealthCheckBatchRule for PendingHealthCheckBatchRequiredKeyIdRule {
    fn check(&self, item: &PendingHealthCheckBatch) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending health check batch.
#[derive(Debug, Clone)]
pub struct PendingHealthCheckBatchValidator {
    pub rules: Vec<Rc<dyn PendingHealthCheckBatchRule>>,
}

impl PendingHealthCheckBatchValidator {
    pub fn validate(&self, item: &PendingHealthCheckBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingHealthCheckBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingHealthCheckBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_health_check_batch_validator() -> PendingHealthCheckBatchValidator {
    PendingHealthCheckBatchValidator {
        rules: vec![
            Rc::new(PendingHealthCheckBatchKeyIdLengthRule { max_length: 255 }) as Rc<dyn PendingHealthCheckBatchRule>,
            Rc::new(PendingHealthCheckBatchRevisionRangeRule { min: 80, max: 1000 }),
            Rc::new(PendingHealthCheckBatchRequiredKeyIdRule {
                message: "pending health check batch needs an entry".to_string(),
            }),
        ],
    }
}
