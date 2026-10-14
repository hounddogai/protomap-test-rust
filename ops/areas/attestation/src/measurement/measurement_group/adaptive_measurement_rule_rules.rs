use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive measurement rule must pass. Check returns its problems, or none.
pub trait AdaptiveMeasurementRuleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveMeasurementRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementRuleUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveMeasurementRuleRule for AdaptiveMeasurementRuleUpdatedAtRangeRule {
    fn check(&self, item: &AdaptiveMeasurementRule) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementRuleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AdaptiveMeasurementRuleRule for AdaptiveMeasurementRuleAllowedNameRule {
    fn check(&self, item: &AdaptiveMeasurementRule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementRuleCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementRuleRule for AdaptiveMeasurementRuleCodeLengthRule {
    fn check(&self, item: &AdaptiveMeasurementRule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementRuleClusterLengthRule {
    pub max_length: i64,
}

impl AdaptiveMeasurementRuleRule for AdaptiveMeasurementRuleClusterLengthRule {
    fn check(&self, item: &AdaptiveMeasurementRule) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive measurement rule.
#[derive(Debug, Clone)]
pub struct AdaptiveMeasurementRuleValidator {
    pub rules: Vec<Rc<dyn AdaptiveMeasurementRuleRule>>,
}

impl AdaptiveMeasurementRuleValidator {
    pub fn validate(&self, item: &AdaptiveMeasurementRule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveMeasurementRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveMeasurementRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_measurement_rule_validator() -> AdaptiveMeasurementRuleValidator {
    AdaptiveMeasurementRuleValidator {
        rules: vec![
            Rc::new(AdaptiveMeasurementRuleUpdatedAtRangeRule { min: 3600, max: 86400 })
                as Rc<dyn AdaptiveMeasurementRuleRule>,
            Rc::new(AdaptiveMeasurementRuleAllowedNameRule {
                allowed: vec!["manual".to_string()],
            }),
            Rc::new(AdaptiveMeasurementRuleCodeLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveMeasurementRuleClusterLengthRule { max_length: 16 }),
        ],
    }
}
