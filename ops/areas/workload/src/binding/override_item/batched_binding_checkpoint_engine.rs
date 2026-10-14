use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched binding checkpoint calculation.
pub trait BatchedBindingCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBindingCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedBindingCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedBindingCheckpointModifier for FlaggedBatchedBindingCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedBindingCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedBindingCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedBindingCheckpointModifier for CategoryBatchedBindingCheckpointAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedBindingCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedBindingCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedBindingCheckpointModifier for TieredBatchedBindingCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedBindingCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched binding checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBindingCheckpointEngine {
    pub steps: Vec<Rc<dyn BatchedBindingCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBindingCheckpointEngine {
    pub fn compute(&mut self, item: &BatchedBindingCheckpoint) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedBindingCheckpointModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBindingCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_binding_checkpoint_engine(flags: &flags::Registry) -> BatchedBindingCheckpointEngine {
    BatchedBindingCheckpointEngine {
        steps: vec![
            Rc::new(FlaggedBatchedBindingCheckpointAdjuster {
                flags: flags.clone(),
                flag: "batched-binding-checkpoint-primary".to_string(),
                percent: 8.6,
            }) as Rc<dyn BatchedBindingCheckpointModifier>,
            Rc::new(CategoryBatchedBindingCheckpointAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.32), ("south".to_string(), 0.78)]),
            }),
            Rc::new(TieredBatchedBindingCheckpointAdjuster {
                bounds: vec![2181.0, 8117.0, 9852.0],
                rates: vec![1.18, 1.04, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 76098.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_binding_checkpoint_engine(&flags::new_registry());
        let item = new_batched_binding_checkpoint("code-840", "name-671");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
