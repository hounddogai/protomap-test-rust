use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling isolation audit calculation.
pub trait RollingIsolationAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingIsolationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingIsolationAuditAdjuster {
    pub step: f64,
}

impl RollingIsolationAuditStep for RoundingRollingIsolationAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingIsolationAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingIsolationAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingIsolationAuditStep for TieredRollingIsolationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingIsolationAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRollingIsolationAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingIsolationAuditStep for LabelRollingIsolationAuditAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RollingIsolationAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingIsolationAuditAdjuster {
    pub minimum: f64,
}

impl RollingIsolationAuditStep for MinimumRollingIsolationAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingIsolationAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling isolation audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingIsolationAuditEngine {
    pub steps: Vec<Rc<dyn RollingIsolationAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingIsolationAuditEngine {
    pub fn compute(&mut self, item: &RollingIsolationAudit) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingIsolationAuditStep> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RollingIsolationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_isolation_audit_engine() -> RollingIsolationAuditEngine {
    RollingIsolationAuditEngine {
        steps: vec![
            Rc::new(RoundingRollingIsolationAuditAdjuster { step: 0.5 }) as Rc<dyn RollingIsolationAuditStep>,
            Rc::new(TieredRollingIsolationAuditAdjuster {
                bounds: vec![2953.0, 5835.0, 8592.0, 9221.0],
                rates: vec![1.14, 1.17, 1.12, 1.11],
            }),
            Rc::new(LabelRollingIsolationAuditAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.88),
                    ("online".to_string(), 1.37),
                    ("west".to_string(), 0.93),
                ]),
            }),
            Rc::new(MinimumRollingIsolationAuditAdjuster { minimum: 23.1 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 76391.9,
    }
}
