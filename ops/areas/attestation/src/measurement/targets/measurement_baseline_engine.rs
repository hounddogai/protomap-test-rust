use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement baseline calculation.
pub trait MeasurementBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementBaselineAdjuster {
    pub minimum: f64,
}

impl MeasurementBaselineFactor for MinimumMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineCeilingOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementBaselineFactor for MeasurementBaselineCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementBaseline, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementBaselineFactor for TieredMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitMeasurementBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementBaselineFactor for UnitMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &MeasurementBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementBaselineEngine {
    pub steps: Vec<Rc<dyn MeasurementBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementBaselineEngine {
    pub fn price(&mut self, item: &MeasurementBaseline) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_measurement_baseline_engine() -> MeasurementBaselineEngine {
    MeasurementBaselineEngine {
        steps: vec![
            Rc::new(MinimumMeasurementBaselineAdjuster { minimum: 6.6 }) as Rc<dyn MeasurementBaselineFactor>,
            Rc::new(MeasurementBaselineCeilingOffsetAdjuster { weight: 1.03 }),
            Rc::new(TieredMeasurementBaselineAdjuster {
                bounds: vec![69.0, 5458.0, 6291.0, 9634.0],
                rates: vec![1.16, 0.8, 0.96, 1.05],
            }),
            Rc::new(UnitMeasurementBaselineAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.27)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 32365.4,
    }
}
