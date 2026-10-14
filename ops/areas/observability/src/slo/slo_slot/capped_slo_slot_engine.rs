use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped slo slot calculation.
pub trait CappedSloSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSloSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedSloSlotAdjuster {
    pub step: f64,
}

impl CappedSloSlotStep for RoundingCappedSloSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedSloSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedSloSlotAdjuster {
    pub cap: f64,
}

impl CappedSloSlotStep for CapCappedSloSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedSloSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedSloSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedSloSlotStep for TitleCappedSloSlotAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedSloSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the capped slo slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSloSlotEngine {
    pub steps: Vec<Rc<dyn CappedSloSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSloSlotEngine {
    pub fn run(&mut self, item: &CappedSloSlot) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CappedSloSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_slo_slot_engine() -> CappedSloSlotEngine {
    CappedSloSlotEngine {
        steps: vec![
            Rc::new(RoundingCappedSloSlotAdjuster { step: 0.5 }) as Rc<dyn CappedSloSlotStep>,
            Rc::new(CapCappedSloSlotAdjuster { cap: 29890.0 }),
            Rc::new(TitleCappedSloSlotAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.2)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 66819.4,
    }
}
