use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink variance calculation.
pub trait SinkVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkVarianceStep for FlaggedSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkVarianceStep for TieredSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkVarianceLimitOffsetAdjuster {
    pub weight: f64,
}

impl SinkVarianceStep for SinkVarianceLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &SinkVariance, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkVarianceAdjuster {
    pub percent: f64,
}

impl SinkVarianceStep for PercentSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkVarianceAdjuster {
    pub cap: f64,
}

impl SinkVarianceStep for CapSinkVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the sink variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkVarianceEngine {
    pub steps: Vec<Rc<dyn SinkVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkVarianceEngine {
    pub fn run(&mut self, item: &SinkVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SinkVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_sink_variance_engine(flags: &flags::Registry) -> SinkVarianceEngine {
    SinkVarianceEngine {
        steps: vec![
            Rc::new(FlaggedSinkVarianceAdjuster {
                flags: flags.clone(),
                flag: "sink-variance-weighted".to_string(),
                percent: -0.2,
            }) as Rc<dyn SinkVarianceStep>,
            Rc::new(TieredSinkVarianceAdjuster {
                bounds: vec![4557.0, 5485.0],
                rates: vec![1.12, 1.07],
            }),
            Rc::new(SinkVarianceLimitOffsetAdjuster { weight: 0.93 }),
            Rc::new(PercentSinkVarianceAdjuster { percent: 21.6 }),
            Rc::new(CapSinkVarianceAdjuster { cap: 5175.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 36652.6,
    }
}
