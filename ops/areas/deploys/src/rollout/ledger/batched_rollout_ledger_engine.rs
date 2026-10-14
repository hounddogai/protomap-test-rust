use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched rollout ledger calculation.
pub trait BatchedRolloutLedgerAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRolloutLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRolloutLedgerAdjuster {
    pub percent: f64,
}

impl BatchedRolloutLedgerAdjuster for PercentBatchedRolloutLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRolloutLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRolloutLedgerStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRolloutLedgerAdjuster for BatchedRolloutLedgerStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRolloutLedger, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceBatchedRolloutLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRolloutLedgerAdjuster for NamespaceBatchedRolloutLedgerAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &BatchedRolloutLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRolloutLedgerAdjuster {
    pub step: f64,
}

impl BatchedRolloutLedgerAdjuster for RoundingBatchedRolloutLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRolloutLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedRolloutLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedRolloutLedgerAdjuster for TieredBatchedRolloutLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedRolloutLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched rollout ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRolloutLedgerEngine {
    pub steps: Vec<Rc<dyn BatchedRolloutLedgerAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRolloutLedgerEngine {
    pub fn price(&mut self, item: &BatchedRolloutLedger) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedRolloutLedgerAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRolloutLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_rollout_ledger_engine() -> BatchedRolloutLedgerEngine {
    BatchedRolloutLedgerEngine {
        steps: vec![
            Rc::new(PercentBatchedRolloutLedgerAdjuster { percent: -4.7 }) as Rc<dyn BatchedRolloutLedgerAdjuster>,
            Rc::new(BatchedRolloutLedgerStartsAtOffsetAdjuster { weight: 1.79 }),
            Rc::new(NamespaceBatchedRolloutLedgerAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.79),
                    ("online".to_string(), 0.92),
                    ("north".to_string(), 0.84),
                ]),
            }),
            Rc::new(RoundingBatchedRolloutLedgerAdjuster { step: 0.05 }),
            Rc::new(TieredBatchedRolloutLedgerAdjuster {
                bounds: vec![2179.0, 4442.0],
                rates: vec![1.11, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 25831.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_rollout_ledger_engine();
        let item = new_batched_rollout_ledger("code-931", "name-284");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
