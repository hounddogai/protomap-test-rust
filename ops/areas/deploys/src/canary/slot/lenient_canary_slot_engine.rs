use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient canary slot calculation.
pub trait LenientCanarySlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientCanarySlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientCanarySlotAdjuster {
    pub cap: f64,
}

impl LenientCanarySlotAdjuster for CapLenientCanarySlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientCanarySlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientCanarySlotAdjuster {
    pub step: f64,
}

impl LenientCanarySlotAdjuster for RoundingLenientCanarySlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientCanarySlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientCanarySlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientCanarySlotAdjuster for LabelLenientCanarySlotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientCanarySlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient canary slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientCanarySlotEngine {
    pub steps: Vec<Rc<dyn LenientCanarySlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientCanarySlotEngine {
    pub fn compute(&mut self, item: &LenientCanarySlot) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientCanarySlotAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientCanarySlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_canary_slot_engine() -> LenientCanarySlotEngine {
    LenientCanarySlotEngine {
        steps: vec![
            Rc::new(CapLenientCanarySlotAdjuster { cap: 8305.0 }) as Rc<dyn LenientCanarySlotAdjuster>,
            Rc::new(RoundingLenientCanarySlotAdjuster { step: 0.05 }),
            Rc::new(LabelLenientCanarySlotAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.86)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 70712.2,
    }
}
