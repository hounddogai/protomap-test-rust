use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding slot calculation.
pub trait BindingSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingSlotAdjuster {
    pub step: f64,
}

impl BindingSlotModifier for RoundingBindingSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBindingSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingSlotModifier for CountryBindingSlotAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BindingSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotMinimumOffsetAdjuster {
    pub weight: f64,
}

impl BindingSlotModifier for BindingSlotMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &BindingSlot, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the binding slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingSlotEngine {
    pub steps: Vec<Rc<dyn BindingSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingSlotEngine {
    pub fn compute(&mut self, item: &BindingSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BindingSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_binding_slot_engine() -> BindingSlotEngine {
    BindingSlotEngine {
        steps: vec![
            Rc::new(RoundingBindingSlotAdjuster { step: 0.05 }) as Rc<dyn BindingSlotModifier>,
            Rc::new(CountryBindingSlotAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.4),
                    ("north".to_string(), 0.91),
                    ("west".to_string(), 0.99),
                ]),
            }),
            Rc::new(BindingSlotMinimumOffsetAdjuster { weight: 1.65 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 7743.1,
    }
}
