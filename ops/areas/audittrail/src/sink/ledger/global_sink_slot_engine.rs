use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global sink slot calculation.
pub trait GlobalSinkSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSinkSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalSinkSlotAdjuster {
    pub minimum: f64,
}

impl GlobalSinkSlotStep for MinimumGlobalSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalSinkSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryGlobalSinkSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalSinkSlotStep for CountryGlobalSinkSlotAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &GlobalSinkSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalSinkSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalSinkSlotStep for TieredGlobalSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalSinkSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalSinkSlotAdjuster {
    pub step: f64,
}

impl GlobalSinkSlotStep for RoundingGlobalSinkSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalSinkSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global sink slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSinkSlotEngine {
    pub steps: Vec<Rc<dyn GlobalSinkSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSinkSlotEngine {
    pub fn compute(&mut self, item: &GlobalSinkSlot) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalSinkSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalSinkSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_sink_slot_engine() -> GlobalSinkSlotEngine {
    GlobalSinkSlotEngine {
        steps: vec![
            Rc::new(MinimumGlobalSinkSlotAdjuster { minimum: 5.0 }) as Rc<dyn GlobalSinkSlotStep>,
            Rc::new(CountryGlobalSinkSlotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.18)]),
            }),
            Rc::new(TieredGlobalSinkSlotAdjuster {
                bounds: vec![166.0, 9849.0],
                rates: vec![0.84, 1.04],
            }),
            Rc::new(RoundingGlobalSinkSlotAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 27117.0,
    }
}
