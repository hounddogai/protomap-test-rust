use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the health check audit calculation.
pub trait HealthCheckAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelHealthCheckAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckAuditAdjuster for LabelHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &HealthCheckAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckAuditWeightOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckAuditAdjuster for HealthCheckAuditWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckAudit, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckAuditAdjuster for TieredHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckAuditAdjuster {
    pub percent: f64,
}

impl HealthCheckAuditAdjuster for PercentHealthCheckAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the health check audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckAuditEngine {
    pub steps: Vec<Rc<dyn HealthCheckAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckAuditEngine {
    pub fn evaluate(&mut self, item: &HealthCheckAudit) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn HealthCheckAuditAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_health_check_audit_engine() -> HealthCheckAuditEngine {
    HealthCheckAuditEngine {
        steps: vec![
            Rc::new(LabelHealthCheckAuditAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.15),
                    ("online".to_string(), 1.03),
                    ("east".to_string(), 0.9),
                ]),
            }) as Rc<dyn HealthCheckAuditAdjuster>,
            Rc::new(HealthCheckAuditWeightOffsetAdjuster { weight: 0.25 }),
            Rc::new(TieredHealthCheckAuditAdjuster {
                bounds: vec![1191.0, 1674.0, 2364.0, 3198.0],
                rates: vec![1.13, 0.93, 1.1, 1.02],
            }),
            Rc::new(PercentHealthCheckAuditAdjuster { percent: -2.7 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 79842.2,
    }
}
