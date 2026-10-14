use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched rollback batch calculation.
pub trait BatchedRollbackBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRollbackBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedRollbackBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedRollbackBatchAdjuster for FlaggedBatchedRollbackBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedRollbackBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedRollbackBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedRollbackBatchAdjuster for TieredBatchedRollbackBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedRollbackBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRollbackBatchDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRollbackBatchAdjuster for BatchedRollbackBatchDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRollbackBatch, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRollbackBatchAdjuster {
    pub step: f64,
}

impl BatchedRollbackBatchAdjuster for RoundingBatchedRollbackBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRollbackBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched rollback batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRollbackBatchEngine {
    pub steps: Vec<Rc<dyn BatchedRollbackBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRollbackBatchEngine {
    pub fn price(&mut self, item: &BatchedRollbackBatch) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedRollbackBatchAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRollbackBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_rollback_batch_engine(flags: &flags::Registry) -> BatchedRollbackBatchEngine {
    BatchedRollbackBatchEngine {
        steps: vec![
            Rc::new(FlaggedBatchedRollbackBatchAdjuster {
                flags: flags.clone(),
                flag: "batched-rollback-batch-rolling".to_string(),
                percent: 6.2,
            }) as Rc<dyn BatchedRollbackBatchAdjuster>,
            Rc::new(TieredBatchedRollbackBatchAdjuster {
                bounds: vec![372.0, 5135.0],
                rates: vec![1.11, 1.02],
            }),
            Rc::new(BatchedRollbackBatchDurationMinutesOffsetAdjuster { weight: 0.5 }),
            Rc::new(RoundingBatchedRollbackBatchAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 73350.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_rollback_batch_engine(&flags::new_registry());
        let item = new_batched_rollback_batch("code-500", "label-79");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
