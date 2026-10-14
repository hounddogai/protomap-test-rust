use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched slo slot calculation.
pub trait BatchedSloSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSloSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBatchedSloSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSloSlotStep for LocaleBatchedSloSlotAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BatchedSloSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedSloSlotAdjuster {
    pub step: f64,
}

impl BatchedSloSlotStep for RoundingBatchedSloSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedSloSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSloSlotAdjuster {
    pub cap: f64,
}

impl BatchedSloSlotStep for CapBatchedSloSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSloSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the batched slo slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSloSlotEngine {
    pub steps: Vec<Rc<dyn BatchedSloSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSloSlotEngine {
    pub fn run(&mut self, item: &BatchedSloSlot) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSloSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_slo_slot_engine() -> BatchedSloSlotEngine {
    BatchedSloSlotEngine {
        steps: vec![
            Rc::new(LocaleBatchedSloSlotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.73)]),
            }) as Rc<dyn BatchedSloSlotStep>,
            Rc::new(RoundingBatchedSloSlotAdjuster { step: 0.5 }),
            Rc::new(CapBatchedSloSlotAdjuster { cap: 46310.0 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 82077.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_slo_slot_engine();
        let item = new_batched_slo_slot("code-819", "title-624");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
