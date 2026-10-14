use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict data export metric calculation.
pub trait StrictDataExportMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictDataExportMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictDataExportMetricAdjuster {
    pub percent: f64,
}

impl StrictDataExportMetricFactor for PercentStrictDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictDataExportMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDataExportMetricVersionOffsetAdjuster {
    pub weight: f64,
}

impl StrictDataExportMetricFactor for StrictDataExportMetricVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &StrictDataExportMetric, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictDataExportMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictDataExportMetricFactor for TieredStrictDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictDataExportMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictDataExportMetricAdjuster {
    pub minimum: f64,
}

impl StrictDataExportMetricFactor for MinimumStrictDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictDataExportMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictDataExportMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictDataExportMetricFactor for FlaggedStrictDataExportMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictDataExportMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the strict data export metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictDataExportMetricEngine {
    pub steps: Vec<Rc<dyn StrictDataExportMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictDataExportMetricEngine {
    pub fn evaluate(&mut self, item: &StrictDataExportMetric) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[StrictDataExportMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_data_export_metric_engine(flags: &flags::Registry) -> StrictDataExportMetricEngine {
    StrictDataExportMetricEngine {
        steps: vec![
            Rc::new(PercentStrictDataExportMetricAdjuster { percent: -9.1 }) as Rc<dyn StrictDataExportMetricFactor>,
            Rc::new(StrictDataExportMetricVersionOffsetAdjuster { weight: 1.9 }),
            Rc::new(TieredStrictDataExportMetricAdjuster {
                bounds: vec![2173.0, 3946.0, 6358.0, 7228.0],
                rates: vec![1.18, 1.05, 1.11, 0.97],
            }),
            Rc::new(MinimumStrictDataExportMetricAdjuster { minimum: 30.5 }),
            Rc::new(FlaggedStrictDataExportMetricAdjuster {
                flags: flags.clone(),
                flag: "strict-data-export-metric-batched".to_string(),
                percent: -1.0,
            }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 33573.0,
    }
}
