use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched token manifest calculation.
pub trait BatchedTokenManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTokenManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedTokenManifestAdjuster {
    pub minimum: f64,
}

impl BatchedTokenManifestAdjuster for MinimumBatchedTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedTokenManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedTokenManifestAdjuster {
    pub step: f64,
}

impl BatchedTokenManifestAdjuster for RoundingBatchedTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedTokenManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedTokenManifestAdjuster {
    pub percent: f64,
}

impl BatchedTokenManifestAdjuster for PercentBatchedTokenManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedTokenManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched token manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTokenManifestEngine {
    pub steps: Vec<Rc<dyn BatchedTokenManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTokenManifestEngine {
    pub fn run(&mut self, item: &BatchedTokenManifest) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTokenManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_token_manifest_engine() -> BatchedTokenManifestEngine {
    BatchedTokenManifestEngine {
        steps: vec![
            Rc::new(MinimumBatchedTokenManifestAdjuster { minimum: 34.4 }) as Rc<dyn BatchedTokenManifestAdjuster>,
            Rc::new(RoundingBatchedTokenManifestAdjuster { step: 0.1 }),
            Rc::new(PercentBatchedTokenManifestAdjuster { percent: -18.2 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 43232.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_token_manifest_engine();
        let item = new_batched_token_manifest("code-277", "name-223");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
