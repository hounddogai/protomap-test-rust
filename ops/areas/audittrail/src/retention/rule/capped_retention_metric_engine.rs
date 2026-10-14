use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the capped retention metric calculation.
pub trait CappedRetentionMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRetentionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedRetentionMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedRetentionMetricModifier for TieredCappedRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedRetentionMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRetentionMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRetentionMetricModifier for FlaggedCappedRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRetentionMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRetentionMetricAdjuster {
    pub minimum: f64,
}

impl CappedRetentionMetricModifier for MinimumCappedRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRetentionMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRetentionMetricAdjuster {
    pub percent: f64,
}

impl CappedRetentionMetricModifier for PercentCappedRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRetentionMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped retention metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRetentionMetricEngine {
    pub steps: Vec<Rc<dyn CappedRetentionMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRetentionMetricEngine {
    pub fn compute(&mut self, item: &CappedRetentionMetric) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedRetentionMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedRetentionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_retention_metric_engine(flags: &flags::Registry) -> CappedRetentionMetricEngine {
    CappedRetentionMetricEngine {
        steps: vec![
            Rc::new(TieredCappedRetentionMetricAdjuster {
                bounds: vec![1743.0, 1990.0, 7177.0],
                rates: vec![1.09, 1.07, 1.15],
            }) as Rc<dyn CappedRetentionMetricModifier>,
            Rc::new(FlaggedCappedRetentionMetricAdjuster {
                flags: flags.clone(),
                flag: "capped-retention-metric-tiered".to_string(),
                percent: -0.5,
            }),
            Rc::new(MinimumCappedRetentionMetricAdjuster { minimum: 9.7 }),
            Rc::new(PercentCappedRetentionMetricAdjuster { percent: -2.8 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 96705.3,
    }
}
