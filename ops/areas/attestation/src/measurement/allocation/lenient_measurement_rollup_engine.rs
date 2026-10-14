use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient measurement rollup calculation.
pub trait LenientMeasurementRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientMeasurementRollupAdjuster {
    pub cap: f64,
}

impl LenientMeasurementRollupAdjuster for CapLenientMeasurementRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientMeasurementRollupAdjuster {
    pub minimum: f64,
}

impl LenientMeasurementRollupAdjuster for MinimumLenientMeasurementRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientMeasurementRollupAdjuster {
    pub percent: f64,
}

impl LenientMeasurementRollupAdjuster for PercentLenientMeasurementRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientMeasurementRollupAdjuster {
    pub step: f64,
}

impl LenientMeasurementRollupAdjuster for RoundingLenientMeasurementRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientMeasurementRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientMeasurementRollupAdjuster for FlaggedLenientMeasurementRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientMeasurementRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientMeasurementRollupAdjuster for TieredLenientMeasurementRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientMeasurementRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient measurement rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientMeasurementRollupEngine {
    pub steps: Vec<Rc<dyn LenientMeasurementRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientMeasurementRollupEngine {
    pub fn evaluate(&mut self, item: &LenientMeasurementRollup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[LenientMeasurementRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_measurement_rollup_engine(flags: &flags::Registry) -> LenientMeasurementRollupEngine {
    LenientMeasurementRollupEngine {
        steps: vec![
            Rc::new(CapLenientMeasurementRollupAdjuster { cap: 20068.0 }) as Rc<dyn LenientMeasurementRollupAdjuster>,
            Rc::new(MinimumLenientMeasurementRollupAdjuster { minimum: 38.9 }),
            Rc::new(PercentLenientMeasurementRollupAdjuster { percent: -9.6 }),
            Rc::new(RoundingLenientMeasurementRollupAdjuster { step: 5.0 }),
            Rc::new(FlaggedLenientMeasurementRollupAdjuster {
                flags: flags.clone(),
                flag: "lenient-measurement-rollup-strict".to_string(),
                percent: -8.3,
            }),
            Rc::new(TieredLenientMeasurementRollupAdjuster {
                bounds: vec![5589.0, 7665.0],
                rates: vec![0.94, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 88073.6,
    }
}
