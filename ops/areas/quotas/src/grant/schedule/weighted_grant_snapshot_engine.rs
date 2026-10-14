use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted grant snapshot calculation.
pub trait WeightedGrantSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedGrantSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotQuantityOffsetAdjuster {
    pub weight: f64,
}

impl WeightedGrantSnapshotFactor for WeightedGrantSnapshotQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedGrantSnapshot, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedGrantSnapshotAdjuster {
    pub minimum: f64,
}

impl WeightedGrantSnapshotFactor for MinimumWeightedGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedGrantSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedGrantSnapshotAdjuster {
    pub step: f64,
}

impl WeightedGrantSnapshotFactor for RoundingWeightedGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedGrantSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedGrantSnapshotAdjuster {
    pub cap: f64,
}

impl WeightedGrantSnapshotFactor for CapWeightedGrantSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedGrantSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted grant snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedGrantSnapshotEngine {
    pub steps: Vec<Rc<dyn WeightedGrantSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedGrantSnapshotEngine {
    pub fn run(&mut self, item: &WeightedGrantSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedGrantSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_grant_snapshot_engine() -> WeightedGrantSnapshotEngine {
    WeightedGrantSnapshotEngine {
        steps: vec![
            Rc::new(WeightedGrantSnapshotQuantityOffsetAdjuster { weight: 0.1 }) as Rc<dyn WeightedGrantSnapshotFactor>,
            Rc::new(MinimumWeightedGrantSnapshotAdjuster { minimum: 24.5 }),
            Rc::new(RoundingWeightedGrantSnapshotAdjuster { step: 5.0 }),
            Rc::new(CapWeightedGrantSnapshotAdjuster { cap: 48907.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 30344.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_grant_snapshot_engine();
        let item = new_weighted_grant_snapshot("code-218", "label-705");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
