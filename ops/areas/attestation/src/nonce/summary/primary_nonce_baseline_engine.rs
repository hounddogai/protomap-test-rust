use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary nonce baseline calculation.
pub trait PrimaryNonceBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryNonceBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryNonceBaselineAdjuster {
    pub minimum: f64,
}

impl PrimaryNonceBaselineStep for MinimumPrimaryNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryNonceBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryNonceBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryNonceBaselineStep for FlaggedPrimaryNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryNonceBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryNonceBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryNonceBaselineStep for TieredPrimaryNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryNonceBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryNonceBaselineAdjuster {
    pub percent: f64,
}

impl PrimaryNonceBaselineStep for PercentPrimaryNonceBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryNonceBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary nonce baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryNonceBaselineEngine {
    pub steps: Vec<Rc<dyn PrimaryNonceBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryNonceBaselineEngine {
    pub fn run(&mut self, item: &PrimaryNonceBaseline) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryNonceBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_nonce_baseline_engine(flags: &flags::Registry) -> PrimaryNonceBaselineEngine {
    PrimaryNonceBaselineEngine {
        steps: vec![
            Rc::new(MinimumPrimaryNonceBaselineAdjuster { minimum: 31.9 }) as Rc<dyn PrimaryNonceBaselineStep>,
            Rc::new(FlaggedPrimaryNonceBaselineAdjuster {
                flags: flags.clone(),
                flag: "primary-nonce-baseline-rolling".to_string(),
                percent: -9.5,
            }),
            Rc::new(TieredPrimaryNonceBaselineAdjuster {
                bounds: vec![120.0, 7667.0, 8987.0],
                rates: vec![1.08, 1.1, 1.17],
            }),
            Rc::new(PercentPrimaryNonceBaselineAdjuster { percent: -17.8 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 13233.9,
    }
}
