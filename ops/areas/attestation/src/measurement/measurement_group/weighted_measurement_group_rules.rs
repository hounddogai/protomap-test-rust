use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted measurement group must pass. Check returns its problems, or none.
pub trait WeightedMeasurementGroupRule: std::fmt::Debug {
    fn check(&self, item: &WeightedMeasurementGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementGroupApprovedGuardRule {
    pub limit: i64,
}

impl WeightedMeasurementGroupRule for WeightedMeasurementGroupApprovedGuardRule {
    fn check(&self, item: &WeightedMeasurementGroup) -> Vec<String> {
        if item.approved && (item.ends_at > self.limit) {
            return vec![format!(
                "ends at {} exceeds {} while approved",
                item.ends_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementGroupRequiredNameRule {
    pub message: String,
}

impl WeightedMeasurementGroupRule for WeightedMeasurementGroupRequiredNameRule {
    fn check(&self, item: &WeightedMeasurementGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementGroupEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedMeasurementGroupRule for WeightedMeasurementGroupEndsAtRangeRule {
    fn check(&self, item: &WeightedMeasurementGroup) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted measurement group.
#[derive(Debug, Clone)]
pub struct WeightedMeasurementGroupValidator {
    pub rules: Vec<Rc<dyn WeightedMeasurementGroupRule>>,
}

impl WeightedMeasurementGroupValidator {
    pub fn validate(&self, item: &WeightedMeasurementGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedMeasurementGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedMeasurementGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_measurement_group_validator() -> WeightedMeasurementGroupValidator {
    WeightedMeasurementGroupValidator {
        rules: vec![
            Rc::new(WeightedMeasurementGroupApprovedGuardRule { limit: 64 }) as Rc<dyn WeightedMeasurementGroupRule>,
            Rc::new(WeightedMeasurementGroupRequiredNameRule {
                message: "weighted measurement group needs an entry".to_string(),
            }),
            Rc::new(WeightedMeasurementGroupEndsAtRangeRule { min: 120, max: 255 }),
        ],
    }
}
