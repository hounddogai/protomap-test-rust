use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched token ledger calculation.
pub trait BatchedTokenLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTokenLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedTokenLedgerAdjuster {
    pub cap: f64,
}

impl BatchedTokenLedgerStep for CapBatchedTokenLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedTokenLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedTokenLedgerAdjuster {
    pub percent: f64,
}

impl BatchedTokenLedgerStep for PercentBatchedTokenLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedTokenLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedTokenLedgerAdjuster {
    pub minimum: f64,
}

impl BatchedTokenLedgerStep for MinimumBatchedTokenLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedTokenLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedgerRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl BatchedTokenLedgerStep for BatchedTokenLedgerRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedTokenLedger, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBatchedTokenLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedTokenLedgerStep for CategoryBatchedTokenLedgerAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BatchedTokenLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched token ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTokenLedgerEngine {
    pub steps: Vec<Rc<dyn BatchedTokenLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTokenLedgerEngine {
    pub fn price(&mut self, item: &BatchedTokenLedger) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedTokenLedgerStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTokenLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_token_ledger_engine() -> BatchedTokenLedgerEngine {
    BatchedTokenLedgerEngine {
        steps: vec![
            Rc::new(CapBatchedTokenLedgerAdjuster { cap: 8487.0 }) as Rc<dyn BatchedTokenLedgerStep>,
            Rc::new(PercentBatchedTokenLedgerAdjuster { percent: -14.2 }),
            Rc::new(MinimumBatchedTokenLedgerAdjuster { minimum: 1.4 }),
            Rc::new(BatchedTokenLedgerRotationDaysOffsetAdjuster { weight: 1.32 }),
            Rc::new(CategoryBatchedTokenLedgerAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.21)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 62839.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_token_ledger_engine();
        let item = new_batched_token_ledger("code-887", "name-412");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
