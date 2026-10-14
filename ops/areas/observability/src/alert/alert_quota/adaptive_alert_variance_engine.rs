use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive alert variance calculation.
pub trait AdaptiveAlertVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveAlertVarianceAdjuster {
    pub step: f64,
}

impl AdaptiveAlertVarianceStep for RoundingAdaptiveAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveAlertVarianceAdjuster {
    pub cap: f64,
}

impl AdaptiveAlertVarianceStep for CapAdaptiveAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveAlertVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveAlertVarianceStep for FlaggedAdaptiveAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveAlertVarianceAdjuster {
    pub percent: f64,
}

impl AdaptiveAlertVarianceStep for PercentAdaptiveAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveAlertVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveAlertVarianceStep for TieredAdaptiveAlertVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAlertVarianceCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveAlertVarianceStep for AdaptiveAlertVarianceCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveAlertVariance, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the adaptive alert variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveAlertVarianceEngine {
    pub steps: Vec<Rc<dyn AdaptiveAlertVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveAlertVarianceEngine {
    pub fn price(&mut self, item: &AdaptiveAlertVariance) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveAlertVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveAlertVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_alert_variance_engine(flags: &flags::Registry) -> AdaptiveAlertVarianceEngine {
    AdaptiveAlertVarianceEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveAlertVarianceAdjuster { step: 1.0 }) as Rc<dyn AdaptiveAlertVarianceStep>,
            Rc::new(CapAdaptiveAlertVarianceAdjuster { cap: 12520.0 }),
            Rc::new(FlaggedAdaptiveAlertVarianceAdjuster {
                flags: flags.clone(),
                flag: "adaptive-alert-variance-regional".to_string(),
                percent: 1.0,
            }),
            Rc::new(PercentAdaptiveAlertVarianceAdjuster { percent: 3.7 }),
            Rc::new(TieredAdaptiveAlertVarianceAdjuster {
                bounds: vec![189.0, 8164.0, 9565.0],
                rates: vec![1.18, 1.19, 0.92],
            }),
            Rc::new(AdaptiveAlertVarianceCreatedAtOffsetAdjuster { weight: 1.6 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 2021.1,
    }
}
