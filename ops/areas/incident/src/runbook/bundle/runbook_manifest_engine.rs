use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook manifest calculation.
pub trait RunbookManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookManifestAdjuster {
    pub step: f64,
}

impl RunbookManifestModifier for RoundingRunbookManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookManifestAdjuster {
    pub minimum: f64,
}

impl RunbookManifestModifier for MinimumRunbookManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookManifestModifier for TieredRunbookManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookManifestAdjuster {
    pub cap: f64,
}

impl RunbookManifestModifier for CapRunbookManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookManifestAmountOffsetAdjuster {
    pub weight: f64,
}

impl RunbookManifestModifier for RunbookManifestAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookManifest, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the runbook manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookManifestEngine {
    pub steps: Vec<Rc<dyn RunbookManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookManifestEngine {
    pub fn price(&mut self, item: &RunbookManifest) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_manifest_engine() -> RunbookManifestEngine {
    RunbookManifestEngine {
        steps: vec![
            Rc::new(RoundingRunbookManifestAdjuster { step: 1.0 }) as Rc<dyn RunbookManifestModifier>,
            Rc::new(MinimumRunbookManifestAdjuster { minimum: 7.2 }),
            Rc::new(TieredRunbookManifestAdjuster {
                bounds: vec![448.0, 6351.0],
                rates: vec![0.98, 1.06],
            }),
            Rc::new(CapRunbookManifestAdjuster { cap: 1157.0 }),
            Rc::new(RunbookManifestAmountOffsetAdjuster { weight: 1.49 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 28820.9,
    }
}
