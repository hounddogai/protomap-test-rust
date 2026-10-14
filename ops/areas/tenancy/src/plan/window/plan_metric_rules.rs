use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a plan metric must pass. Check returns its problems, or none.
pub trait PlanMetricRule: std::fmt::Debug {
    fn check(&self, item: &PlanMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetricRequiredOwnerIdRule {
    pub message: String,
}

impl PlanMetricRule for PlanMetricRequiredOwnerIdRule {
    fn check(&self, item: &PlanMetric) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetricCodePrefixRule {
    pub prefix: String,
}

impl PlanMetricRule for PlanMetricCodePrefixRule {
    fn check(&self, item: &PlanMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetricCodeLengthRule {
    pub max_length: i64,
}

impl PlanMetricRule for PlanMetricCodeLengthRule {
    fn check(&self, item: &PlanMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetricOwnerIdLengthRule {
    pub max_length: i64,
}

impl PlanMetricRule for PlanMetricOwnerIdLengthRule {
    fn check(&self, item: &PlanMetric) -> Vec<String> {
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
pub struct PlanMetricRequiredCodeRule {
    pub message: String,
}

impl PlanMetricRule for PlanMetricRequiredCodeRule {
    fn check(&self, item: &PlanMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanMetricRequiredKeyIdRule {
    pub message: String,
}

impl PlanMetricRule for PlanMetricRequiredKeyIdRule {
    fn check(&self, item: &PlanMetric) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a plan metric.
#[derive(Debug, Clone)]
pub struct PlanMetricValidator {
    pub rules: Vec<Rc<dyn PlanMetricRule>>,
}

impl PlanMetricValidator {
    pub fn validate(&self, item: &PlanMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PlanMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_metric_validator() -> PlanMetricValidator {
    PlanMetricValidator {
        rules: vec![
            Rc::new(PlanMetricRequiredOwnerIdRule {
                message: "plan metric needs a value".to_string(),
            }) as Rc<dyn PlanMetricRule>,
            Rc::new(PlanMetricCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PlanMetricCodeLengthRule { max_length: 80 }),
            Rc::new(PlanMetricOwnerIdLengthRule { max_length: 86400 }),
            Rc::new(PlanMetricRequiredCodeRule {
                message: "plan metric needs an entry".to_string(),
            }),
            Rc::new(PlanMetricRequiredKeyIdRule {
                message: "plan metric needs this field".to_string(),
            }),
        ],
    }
}
