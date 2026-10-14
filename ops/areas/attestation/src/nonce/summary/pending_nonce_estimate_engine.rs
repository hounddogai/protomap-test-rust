use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending nonce estimate calculation.
pub trait PendingNonceEstimateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingNonceEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceEstimateWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PendingNonceEstimateModifier for PendingNonceEstimateWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PendingNonceEstimate, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingNonceEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingNonceEstimateModifier for FlaggedPendingNonceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingNonceEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingNonceEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingNonceEstimateModifier for TieredPendingNonceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingNonceEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingNonceEstimateAdjuster {
    pub minimum: f64,
}

impl PendingNonceEstimateModifier for MinimumPendingNonceEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingNonceEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending nonce estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingNonceEstimateEngine {
    pub steps: Vec<Rc<dyn PendingNonceEstimateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingNonceEstimateEngine {
    pub fn compute(&mut self, item: &PendingNonceEstimate) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingNonceEstimateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingNonceEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_nonce_estimate_engine(flags: &flags::Registry) -> PendingNonceEstimateEngine {
    PendingNonceEstimateEngine {
        steps: vec![
            Rc::new(PendingNonceEstimateWindowDaysOffsetAdjuster { weight: 1.85 })
                as Rc<dyn PendingNonceEstimateModifier>,
            Rc::new(FlaggedPendingNonceEstimateAdjuster {
                flags: flags.clone(),
                flag: "pending-nonce-estimate-default".to_string(),
                percent: 0.9,
            }),
            Rc::new(TieredPendingNonceEstimateAdjuster {
                bounds: vec![1410.0, 1516.0, 5681.0, 9678.0],
                rates: vec![0.81, 0.97, 0.83, 1.12],
            }),
            Rc::new(MinimumPendingNonceEstimateAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 66942.2,
    }
}
