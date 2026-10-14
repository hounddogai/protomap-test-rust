use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the legacy rollout metric calculation.
pub trait LegacyRolloutMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyRolloutMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetricFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl LegacyRolloutMetricAdjuster for LegacyRolloutMetricFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyRolloutMetric, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyRolloutMetricAdjuster {
    pub cap: f64,
}

impl LegacyRolloutMetricAdjuster for CapLegacyRolloutMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyRolloutMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyRolloutMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyRolloutMetricAdjuster for TieredLegacyRolloutMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyRolloutMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy rollout metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyRolloutMetricEngine {
    pub steps: Vec<Rc<dyn LegacyRolloutMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyRolloutMetricEngine {
    pub fn price(&mut self, item: &LegacyRolloutMetric) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyRolloutMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyRolloutMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_rollout_metric_engine() -> LegacyRolloutMetricEngine {
    LegacyRolloutMetricEngine {
        steps: vec![
            Rc::new(LegacyRolloutMetricFloorValueOffsetAdjuster { weight: 0.52 })
                as Rc<dyn LegacyRolloutMetricAdjuster>,
            Rc::new(CapLegacyRolloutMetricAdjuster { cap: 33944.0 }),
            Rc::new(TieredLegacyRolloutMetricAdjuster {
                bounds: vec![123.0, 8354.0],
                rates: vec![0.93, 0.95],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 2701.9,
    }
}
