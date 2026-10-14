use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a health check baseline must pass. Check returns its problems, or none.
pub trait HealthCheckBaselineRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBaselineClusterLengthRule {
    pub max_length: i64,
}

impl HealthCheckBaselineRule for HealthCheckBaselineClusterLengthRule {
    fn check(&self, item: &HealthCheckBaseline) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBaselineTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl HealthCheckBaselineRule for HealthCheckBaselineTtlSecondsRangeRule {
    fn check(&self, item: &HealthCheckBaseline) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBaselineRequiredClusterRule {
    pub message: String,
}

impl HealthCheckBaselineRule for HealthCheckBaselineRequiredClusterRule {
    fn check(&self, item: &HealthCheckBaseline) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBaselineRequiredNameRule {
    pub message: String,
}

impl HealthCheckBaselineRule for HealthCheckBaselineRequiredNameRule {
    fn check(&self, item: &HealthCheckBaseline) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBaselineCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckBaselineRule for HealthCheckBaselineCodePrefixRule {
    fn check(&self, item: &HealthCheckBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check baseline.
#[derive(Debug, Clone)]
pub struct HealthCheckBaselineValidator {
    pub rules: Vec<Rc<dyn HealthCheckBaselineRule>>,
}

impl HealthCheckBaselineValidator {
    pub fn validate(&self, item: &HealthCheckBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_baseline_validator() -> HealthCheckBaselineValidator {
    HealthCheckBaselineValidator {
        rules: vec![
            Rc::new(HealthCheckBaselineClusterLengthRule { max_length: 120 }) as Rc<dyn HealthCheckBaselineRule>,
            Rc::new(HealthCheckBaselineTtlSecondsRangeRule { min: 1000, max: 86400 }),
            Rc::new(HealthCheckBaselineRequiredClusterRule {
                message: "health check baseline needs this field".to_string(),
            }),
            Rc::new(HealthCheckBaselineRequiredNameRule {
                message: "health check baseline needs an entry".to_string(),
            }),
            Rc::new(HealthCheckBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}
