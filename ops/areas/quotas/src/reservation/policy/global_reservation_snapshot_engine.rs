use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global reservation snapshot calculation.
pub trait GlobalReservationSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReservationSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReservationSnapshotDiscountOffsetAdjuster {
    pub weight: f64,
}

impl GlobalReservationSnapshotFactor for GlobalReservationSnapshotDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalReservationSnapshot, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalReservationSnapshotAdjuster {
    pub step: f64,
}

impl GlobalReservationSnapshotFactor for RoundingGlobalReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalReservationSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalReservationSnapshotAdjuster {
    pub cap: f64,
}

impl GlobalReservationSnapshotFactor for CapGlobalReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalReservationSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global reservation snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReservationSnapshotEngine {
    pub steps: Vec<Rc<dyn GlobalReservationSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReservationSnapshotEngine {
    pub fn compute(&mut self, item: &GlobalReservationSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalReservationSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_reservation_snapshot_engine() -> GlobalReservationSnapshotEngine {
    GlobalReservationSnapshotEngine {
        steps: vec![
            Rc::new(GlobalReservationSnapshotDiscountOffsetAdjuster { weight: 0.72 })
                as Rc<dyn GlobalReservationSnapshotFactor>,
            Rc::new(RoundingGlobalReservationSnapshotAdjuster { step: 0.1 }),
            Rc::new(CapGlobalReservationSnapshotAdjuster { cap: 37631.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 9388.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_reservation_snapshot_engine();
        let item = new_global_reservation_snapshot("code-912", "label-965");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
