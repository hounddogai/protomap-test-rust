use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled probe mapping must pass. Check returns its problems, or none.
pub trait ScheduledProbeMappingRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingRequiredCodeRule {
    pub message: String,
}

impl ScheduledProbeMappingRule for ScheduledProbeMappingRequiredCodeRule {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledProbeMappingRule for ScheduledProbeMappingQuantityRangeRule {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingRequiredTitleRule {
    pub message: String,
}

impl ScheduledProbeMappingRule for ScheduledProbeMappingRequiredTitleRule {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledProbeMappingRule for ScheduledProbeMappingTitleLengthRule {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingRequiredHostRule {
    pub message: String,
}

impl ScheduledProbeMappingRule for ScheduledProbeMappingRequiredHostRule {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingRequiredRegionRule {
    pub message: String,
}

impl ScheduledProbeMappingRule for ScheduledProbeMappingRequiredRegionRule {
    fn check(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled probe mapping.
#[derive(Debug, Clone)]
pub struct ScheduledProbeMappingValidator {
    pub rules: Vec<Rc<dyn ScheduledProbeMappingRule>>,
}

impl ScheduledProbeMappingValidator {
    pub fn validate(&self, item: &ScheduledProbeMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledProbeMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledProbeMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_probe_mapping_validator() -> ScheduledProbeMappingValidator {
    ScheduledProbeMappingValidator {
        rules: vec![
            Rc::new(ScheduledProbeMappingRequiredCodeRule {
                message: "scheduled probe mapping needs this field".to_string(),
            }) as Rc<dyn ScheduledProbeMappingRule>,
            Rc::new(ScheduledProbeMappingQuantityRangeRule { min: 16, max: 120 }),
            Rc::new(ScheduledProbeMappingRequiredTitleRule {
                message: "scheduled probe mapping needs this field".to_string(),
            }),
            Rc::new(ScheduledProbeMappingTitleLengthRule { max_length: 120 }),
            Rc::new(ScheduledProbeMappingRequiredHostRule {
                message: "scheduled probe mapping needs this field".to_string(),
            }),
            Rc::new(ScheduledProbeMappingRequiredRegionRule {
                message: "scheduled probe mapping needs a value".to_string(),
            }),
        ],
    }
}
