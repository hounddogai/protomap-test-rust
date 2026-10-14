use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier slot calculation.
pub trait VerifierSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierSlotAdjuster {
    pub cap: f64,
}

impl VerifierSlotModifier for CapVerifierSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSlotGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl VerifierSlotModifier for VerifierSlotGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierSlotModifier for TieredVerifierSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierSlotModifier for FlaggedVerifierSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierSlotAdjuster {
    pub minimum: f64,
}

impl VerifierSlotModifier for MinimumVerifierSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentVerifierSlotAdjuster {
    pub percent: f64,
}

impl VerifierSlotModifier for PercentVerifierSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &VerifierSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the verifier slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierSlotEngine {
    pub steps: Vec<Rc<dyn VerifierSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierSlotEngine {
    pub fn price(&mut self, item: &VerifierSlot) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[VerifierSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_verifier_slot_engine(flags: &flags::Registry) -> VerifierSlotEngine {
    VerifierSlotEngine {
        steps: vec![
            Rc::new(CapVerifierSlotAdjuster { cap: 46356.0 }) as Rc<dyn VerifierSlotModifier>,
            Rc::new(VerifierSlotGraceDaysOffsetAdjuster { weight: 1.34 }),
            Rc::new(TieredVerifierSlotAdjuster {
                bounds: vec![219.0, 260.0, 4300.0],
                rates: vec![1.07, 1.02, 1.16],
            }),
            Rc::new(FlaggedVerifierSlotAdjuster {
                flags: flags.clone(),
                flag: "verifier-slot-rolling".to_string(),
                percent: -5.5,
            }),
            Rc::new(MinimumVerifierSlotAdjuster { minimum: 4.8 }),
            Rc::new(PercentVerifierSlotAdjuster { percent: -23.2 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 26744.6,
    }
}
