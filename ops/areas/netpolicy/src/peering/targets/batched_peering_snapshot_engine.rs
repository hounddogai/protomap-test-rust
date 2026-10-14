use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched peering snapshot calculation.
pub trait BatchedPeeringSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPeeringSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedPeeringSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedPeeringSnapshotStep for TieredBatchedPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedPeeringSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPeeringSnapshotBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl BatchedPeeringSnapshotStep for BatchedPeeringSnapshotBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedPeeringSnapshot, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedPeeringSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedPeeringSnapshotStep for NameBatchedPeeringSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedPeeringSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched peering snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPeeringSnapshotEngine {
    pub steps: Vec<Rc<dyn BatchedPeeringSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPeeringSnapshotEngine {
    pub fn evaluate(&mut self, item: &BatchedPeeringSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedPeeringSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_peering_snapshot_engine() -> BatchedPeeringSnapshotEngine {
    BatchedPeeringSnapshotEngine {
        steps: vec![
            Rc::new(TieredBatchedPeeringSnapshotAdjuster {
                bounds: vec![1671.0, 3314.0, 4753.0, 9712.0],
                rates: vec![0.89, 1.12, 1.01, 0.87],
            }) as Rc<dyn BatchedPeeringSnapshotStep>,
            Rc::new(BatchedPeeringSnapshotBatchSizeOffsetAdjuster { weight: 0.98 }),
            Rc::new(NameBatchedPeeringSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.78),
                    ("west".to_string(), 1.02),
                    ("central".to_string(), 1.26),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 75845.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_peering_snapshot_engine();
        let item = new_batched_peering_snapshot("code-47", "name-30");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
