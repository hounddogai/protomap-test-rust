use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation rollup must pass. Check returns its problems, or none.
pub trait ReservationRollupRule: std::fmt::Debug {
    fn check(&self, item: &ReservationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationRollupRequiredLabelRule {
    pub message: String,
}

impl ReservationRollupRule for ReservationRollupRequiredLabelRule {
    fn check(&self, item: &ReservationRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationRollupThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationRollupRule for ReservationRollupThresholdRangeRule {
    fn check(&self, item: &ReservationRollup) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationRollupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReservationRollupRule for ReservationRollupTtlSecondsRangeRule {
    fn check(&self, item: &ReservationRollup) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a reservation rollup.
#[derive(Debug, Clone)]
pub struct ReservationRollupValidator {
    pub rules: Vec<Rc<dyn ReservationRollupRule>>,
}

impl ReservationRollupValidator {
    pub fn validate(&self, item: &ReservationRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_rollup_validator() -> ReservationRollupValidator {
    ReservationRollupValidator {
        rules: vec![
            Rc::new(ReservationRollupRequiredLabelRule {
                message: "reservation rollup needs this field".to_string(),
            }) as Rc<dyn ReservationRollupRule>,
            Rc::new(ReservationRollupThresholdRangeRule { min: 2.4, max: 1975.9 }),
            Rc::new(ReservationRollupTtlSecondsRangeRule { min: 16, max: 16 }),
        ],
    }
}
