use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global silence target calculation.
pub trait GlobalSilenceTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalSilenceTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalSilenceTargetAdjuster for TieredGlobalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameGlobalSilenceTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalSilenceTargetAdjuster for NameGlobalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTargetCapacityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalSilenceTargetAdjuster for GlobalSilenceTargetCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalSilenceTargetAdjuster {
    pub cap: f64,
}

impl GlobalSilenceTargetAdjuster for CapGlobalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalSilenceTargetAdjuster {
    pub minimum: f64,
}

impl GlobalSilenceTargetAdjuster for MinimumGlobalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalSilenceTargetAdjuster {
    pub step: f64,
}

impl GlobalSilenceTargetAdjuster for RoundingGlobalSilenceTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalSilenceTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global silence target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSilenceTargetEngine {
    pub steps: Vec<Rc<dyn GlobalSilenceTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSilenceTargetEngine {
    pub fn price(&mut self, item: &GlobalSilenceTarget) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalSilenceTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalSilenceTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_silence_target_engine() -> GlobalSilenceTargetEngine {
    GlobalSilenceTargetEngine {
        steps: vec![
            Rc::new(TieredGlobalSilenceTargetAdjuster {
                bounds: vec![7426.0, 7642.0],
                rates: vec![0.83, 0.86],
            }) as Rc<dyn GlobalSilenceTargetAdjuster>,
            Rc::new(NameGlobalSilenceTargetAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.37),
                    ("online".to_string(), 1.1),
                    ("north".to_string(), 1.04),
                ]),
            }),
            Rc::new(GlobalSilenceTargetCapacityOffsetAdjuster { weight: 0.9 }),
            Rc::new(CapGlobalSilenceTargetAdjuster { cap: 25695.0 }),
            Rc::new(MinimumGlobalSilenceTargetAdjuster { minimum: 12.5 }),
            Rc::new(RoundingGlobalSilenceTargetAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 18068.8,
    }
}
