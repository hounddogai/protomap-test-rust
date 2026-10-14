use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback grant agreement calculation.
pub trait FallbackGrantAgreementAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackGrantAgreementAdjuster {
    pub step: f64,
}

impl FallbackGrantAgreementAdjuster for RoundingFallbackGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackGrantAgreementAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackGrantAgreementAdjuster for FlaggedFallbackGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackGrantAgreementAdjuster {
    pub percent: f64,
}

impl FallbackGrantAgreementAdjuster for PercentFallbackGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackGrantAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackGrantAgreementAdjuster for TieredFallbackGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackGrantAgreementAdjuster {
    pub cap: f64,
}

impl FallbackGrantAgreementAdjuster for CapFallbackGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackGrantAgreementBurstOffsetAdjuster {
    pub weight: f64,
}

impl FallbackGrantAgreementAdjuster for FallbackGrantAgreementBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackGrantAgreement, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

/// Runs the fallback grant agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackGrantAgreementEngine {
    pub steps: Vec<Rc<dyn FallbackGrantAgreementAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackGrantAgreementEngine {
    pub fn compute(&mut self, item: &FallbackGrantAgreement) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackGrantAgreementAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackGrantAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_grant_agreement_engine(flags: &flags::Registry) -> FallbackGrantAgreementEngine {
    FallbackGrantAgreementEngine {
        steps: vec![
            Rc::new(RoundingFallbackGrantAgreementAdjuster { step: 0.05 }) as Rc<dyn FallbackGrantAgreementAdjuster>,
            Rc::new(FlaggedFallbackGrantAgreementAdjuster {
                flags: flags.clone(),
                flag: "fallback-grant-agreement-strict".to_string(),
                percent: 7.6,
            }),
            Rc::new(PercentFallbackGrantAgreementAdjuster { percent: 0.1 }),
            Rc::new(TieredFallbackGrantAgreementAdjuster {
                bounds: vec![4168.0, 7678.0, 8729.0],
                rates: vec![0.91, 1.05, 1.02],
            }),
            Rc::new(CapFallbackGrantAgreementAdjuster { cap: 37635.0 }),
            Rc::new(FallbackGrantAgreementBurstOffsetAdjuster { weight: 0.7 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 31066.1,
    }
}
