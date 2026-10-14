use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default escalation rollup calculation.
pub trait DefaultEscalationRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultEscalationRollupAdjuster {
    pub percent: f64,
}

impl DefaultEscalationRollupFactor for PercentDefaultEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEscalationRollupWeightOffsetAdjuster {
    pub weight: f64,
}

impl DefaultEscalationRollupFactor for DefaultEscalationRollupWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultEscalationRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultEscalationRollupFactor for TieredDefaultEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultEscalationRollupAdjuster {
    pub step: f64,
}

impl DefaultEscalationRollupFactor for RoundingDefaultEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultEscalationRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultEscalationRollupFactor for FlaggedDefaultEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultEscalationRollupAdjuster {
    pub cap: f64,
}

impl DefaultEscalationRollupFactor for CapDefaultEscalationRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultEscalationRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default escalation rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultEscalationRollupEngine {
    pub steps: Vec<Rc<dyn DefaultEscalationRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultEscalationRollupEngine {
    pub fn price(&mut self, item: &DefaultEscalationRollup) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultEscalationRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultEscalationRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_escalation_rollup_engine(flags: &flags::Registry) -> DefaultEscalationRollupEngine {
    DefaultEscalationRollupEngine {
        steps: vec![
            Rc::new(PercentDefaultEscalationRollupAdjuster { percent: -21.5 }) as Rc<dyn DefaultEscalationRollupFactor>,
            Rc::new(DefaultEscalationRollupWeightOffsetAdjuster { weight: 0.17 }),
            Rc::new(TieredDefaultEscalationRollupAdjuster {
                bounds: vec![778.0, 6383.0, 6682.0],
                rates: vec![1.06, 0.95, 1.05],
            }),
            Rc::new(RoundingDefaultEscalationRollupAdjuster { step: 1.0 }),
            Rc::new(FlaggedDefaultEscalationRollupAdjuster {
                flags: flags.clone(),
                flag: "default-escalation-rollup-pending".to_string(),
                percent: -2.6,
            }),
            Rc::new(CapDefaultEscalationRollupAdjuster { cap: 29274.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 37101.9,
    }
}
