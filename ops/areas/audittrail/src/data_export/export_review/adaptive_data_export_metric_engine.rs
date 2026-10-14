use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive data export metric calculation.
pub trait AdaptiveDataExportMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveDataExportMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveDataExportMetricAdjuster {
    pub percent: f64,
}

impl AdaptiveDataExportMetricFactor for PercentAdaptiveDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveDataExportMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveDataExportMetricAdjuster {
    pub minimum: f64,
}

impl AdaptiveDataExportMetricFactor for MinimumAdaptiveDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveDataExportMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveDataExportMetricAdjuster {
    pub cap: f64,
}

impl AdaptiveDataExportMetricFactor for CapAdaptiveDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveDataExportMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive data export metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveDataExportMetricEngine {
    pub steps: Vec<Rc<dyn AdaptiveDataExportMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveDataExportMetricEngine {
    pub fn price(&mut self, item: &AdaptiveDataExportMetric) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveDataExportMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveDataExportMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_data_export_metric_engine() -> AdaptiveDataExportMetricEngine {
    AdaptiveDataExportMetricEngine {
        steps: vec![
            Rc::new(PercentAdaptiveDataExportMetricAdjuster { percent: 19.0 })
                as Rc<dyn AdaptiveDataExportMetricFactor>,
            Rc::new(MinimumAdaptiveDataExportMetricAdjuster { minimum: 49.5 }),
            Rc::new(CapAdaptiveDataExportMetricAdjuster { cap: 24692.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 53975.1,
    }
}
