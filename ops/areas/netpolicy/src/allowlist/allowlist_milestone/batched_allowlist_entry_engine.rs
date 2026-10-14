use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched allowlist entry calculation.
pub trait BatchedAllowlistEntryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAllowlistEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedAllowlistEntryAdjuster {
    pub cap: f64,
}

impl BatchedAllowlistEntryStep for CapBatchedAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedAllowlistEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedAllowlistEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedAllowlistEntryStep for TieredBatchedAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedAllowlistEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistEntryExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAllowlistEntryStep for BatchedAllowlistEntryExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAllowlistEntry, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedAllowlistEntryAdjuster {
    pub percent: f64,
}

impl BatchedAllowlistEntryStep for PercentBatchedAllowlistEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedAllowlistEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched allowlist entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistEntryEngine {
    pub steps: Vec<Rc<dyn BatchedAllowlistEntryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAllowlistEntryEngine {
    pub fn price(&mut self, item: &BatchedAllowlistEntry) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedAllowlistEntryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAllowlistEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_allowlist_entry_engine() -> BatchedAllowlistEntryEngine {
    BatchedAllowlistEntryEngine {
        steps: vec![
            Rc::new(CapBatchedAllowlistEntryAdjuster { cap: 17753.0 }) as Rc<dyn BatchedAllowlistEntryStep>,
            Rc::new(TieredBatchedAllowlistEntryAdjuster {
                bounds: vec![1723.0, 5394.0, 7094.0],
                rates: vec![1.07, 1.1, 1.03],
            }),
            Rc::new(BatchedAllowlistEntryExpiresAtOffsetAdjuster { weight: 1.57 }),
            Rc::new(PercentBatchedAllowlistEntryAdjuster { percent: -24.6 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 35734.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_allowlist_entry_engine();
        let item = new_batched_allowlist_entry("code-279", "label-278");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
