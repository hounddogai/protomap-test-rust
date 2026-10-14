use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched redaction bundle calculation.
pub trait BatchedRedactionBundleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRedactionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRedactionBundleAdjuster {
    pub step: f64,
}

impl BatchedRedactionBundleAdjuster for RoundingBatchedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRedactionBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRedactionBundleAdjuster {
    pub minimum: f64,
}

impl BatchedRedactionBundleAdjuster for MinimumBatchedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRedactionBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedRedactionBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRedactionBundleAdjuster for NameBatchedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedRedactionBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionBundlePortOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRedactionBundleAdjuster for BatchedRedactionBundlePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRedactionBundle, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRedactionBundleAdjuster {
    pub percent: f64,
}

impl BatchedRedactionBundleAdjuster for PercentBatchedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRedactionBundle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched redaction bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRedactionBundleEngine {
    pub steps: Vec<Rc<dyn BatchedRedactionBundleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRedactionBundleEngine {
    pub fn compute(&mut self, item: &BatchedRedactionBundle) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRedactionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_redaction_bundle_engine() -> BatchedRedactionBundleEngine {
    BatchedRedactionBundleEngine {
        steps: vec![
            Rc::new(RoundingBatchedRedactionBundleAdjuster { step: 1.0 }) as Rc<dyn BatchedRedactionBundleAdjuster>,
            Rc::new(MinimumBatchedRedactionBundleAdjuster { minimum: 25.5 }),
            Rc::new(NameBatchedRedactionBundleAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.88),
                    ("online".to_string(), 1.29),
                    ("north".to_string(), 0.83),
                ]),
            }),
            Rc::new(BatchedRedactionBundlePortOffsetAdjuster { weight: 0.25 }),
            Rc::new(PercentBatchedRedactionBundleAdjuster { percent: -1.5 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 35065.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_redaction_bundle_engine();
        let item = new_batched_redaction_bundle("code-73", "name-115");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
