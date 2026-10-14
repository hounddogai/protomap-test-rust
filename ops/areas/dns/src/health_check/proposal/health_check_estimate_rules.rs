use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check estimate must pass. Check returns its problems, or none.
pub trait HealthCheckEstimateRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEstimateRequiredUnitRule {
    pub message: String,
}

impl HealthCheckEstimateRule for HealthCheckEstimateRequiredUnitRule {
    fn check(&self, item: &HealthCheckEstimate) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEstimateRequiredLabelRule {
    pub message: String,
}

impl HealthCheckEstimateRule for HealthCheckEstimateRequiredLabelRule {
    fn check(&self, item: &HealthCheckEstimate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckEstimateRequiredNameRule {
    pub message: String,
}

impl HealthCheckEstimateRule for HealthCheckEstimateRequiredNameRule {
    fn check(&self, item: &HealthCheckEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check estimate.
#[derive(Debug, Clone)]
pub struct HealthCheckEstimateValidator {
    pub rules: Vec<Rc<dyn HealthCheckEstimateRule>>,
}

impl HealthCheckEstimateValidator {
    pub fn validate(&self, item: &HealthCheckEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_estimate_validator() -> HealthCheckEstimateValidator {
    HealthCheckEstimateValidator {
        rules: vec![
            Rc::new(HealthCheckEstimateRequiredUnitRule {
                message: "health check estimate needs this field".to_string(),
            }) as Rc<dyn HealthCheckEstimateRule>,
            Rc::new(HealthCheckEstimateRequiredLabelRule {
                message: "health check estimate needs a value".to_string(),
            }),
            Rc::new(HealthCheckEstimateRequiredNameRule {
                message: "health check estimate needs an entry".to_string(),
            }),
        ],
    }
}
