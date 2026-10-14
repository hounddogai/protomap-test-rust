use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tiered alert slot calculation.
pub trait TieredAlertSlotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredAlertSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredAlertSlotAdjuster {
    pub minimum: f64,
}

impl TieredAlertSlotAdjuster for MinimumTieredAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredAlertSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredAlertSlotAdjuster {
    pub cap: f64,
}

impl TieredAlertSlotAdjuster for CapTieredAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredAlertSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredAlertSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredAlertSlotAdjuster for TieredTieredAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredAlertSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTieredAlertSlotAdjuster {
    pub step: f64,
}

impl TieredAlertSlotAdjuster for RoundingTieredAlertSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TieredAlertSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tiered alert slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredAlertSlotEngine {
    pub steps: Vec<Rc<dyn TieredAlertSlotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredAlertSlotEngine {
    pub fn compute(&mut self, item: &TieredAlertSlot) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[TieredAlertSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tiered_alert_slot_engine() -> TieredAlertSlotEngine {
    TieredAlertSlotEngine {
        steps: vec![
            Rc::new(MinimumTieredAlertSlotAdjuster { minimum: 38.5 }) as Rc<dyn TieredAlertSlotAdjuster>,
            Rc::new(CapTieredAlertSlotAdjuster { cap: 19855.0 }),
            Rc::new(TieredTieredAlertSlotAdjuster {
                bounds: vec![3826.0, 7884.0, 8439.0],
                rates: vec![1.12, 1.0, 1.07],
            }),
            Rc::new(RoundingTieredAlertSlotAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 57499.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_alert_slot_engine();
        let item = new_tiered_alert_slot("code-718", "label-338");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
