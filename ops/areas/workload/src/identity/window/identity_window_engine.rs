use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the identity window calculation.
pub trait IdentityWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityWindowModifier for TieredIdentityWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityWindowAdjuster {
    pub cap: f64,
}

impl IdentityWindowModifier for CapIdentityWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIdentityWindowAdjuster {
    pub step: f64,
}

impl IdentityWindowModifier for RoundingIdentityWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IdentityWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the identity window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityWindowEngine {
    pub steps: Vec<Rc<dyn IdentityWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityWindowEngine {
    pub fn evaluate(&mut self, item: &IdentityWindow) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IdentityWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IdentityWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_identity_window_engine() -> IdentityWindowEngine {
    IdentityWindowEngine {
        steps: vec![
            Rc::new(TieredIdentityWindowAdjuster {
                bounds: vec![172.0, 4348.0],
                rates: vec![1.03, 0.86],
            }) as Rc<dyn IdentityWindowModifier>,
            Rc::new(CapIdentityWindowAdjuster { cap: 40915.0 }),
            Rc::new(RoundingIdentityWindowAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 53984.6,
    }
}
