use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending page metric calculation.
pub trait PendingPageMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingPageMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingPageMetricAdjuster {
    pub cap: f64,
}

impl PendingPageMetricFactor for CapPendingPageMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingPageMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingPageMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingPageMetricFactor for TieredPendingPageMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingPageMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalePendingPageMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingPageMetricFactor for LocalePendingPageMetricAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &PendingPageMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending page metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingPageMetricEngine {
    pub steps: Vec<Rc<dyn PendingPageMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingPageMetricEngine {
    pub fn compute(&mut self, item: &PendingPageMetric) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingPageMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingPageMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_page_metric_engine() -> PendingPageMetricEngine {
    PendingPageMetricEngine {
        steps: vec![
            Rc::new(CapPendingPageMetricAdjuster { cap: 12068.0 }) as Rc<dyn PendingPageMetricFactor>,
            Rc::new(TieredPendingPageMetricAdjuster {
                bounds: vec![1941.0, 3237.0],
                rates: vec![0.94, 1.06],
            }),
            Rc::new(LocalePendingPageMetricAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.06), ("south".to_string(), 1.11)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 19027.8,
    }
}
