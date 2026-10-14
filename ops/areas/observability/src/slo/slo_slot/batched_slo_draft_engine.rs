use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the batched slo draft calculation.
pub trait BatchedSloDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSloDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSloDraftAdjuster {
    pub cap: f64,
}

impl BatchedSloDraftStep for CapBatchedSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSloDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterBatchedSloDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSloDraftStep for ClusterBatchedSloDraftAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &BatchedSloDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedSloDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedSloDraftStep for TieredBatchedSloDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedSloDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched slo draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSloDraftEngine {
    pub steps: Vec<Rc<dyn BatchedSloDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSloDraftEngine {
    pub fn run(&mut self, item: &BatchedSloDraft) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedSloDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSloDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_slo_draft_engine() -> BatchedSloDraftEngine {
    BatchedSloDraftEngine {
        steps: vec![
            Rc::new(CapBatchedSloDraftAdjuster { cap: 41618.0 }) as Rc<dyn BatchedSloDraftStep>,
            Rc::new(ClusterBatchedSloDraftAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.11)]),
            }),
            Rc::new(TieredBatchedSloDraftAdjuster {
                bounds: vec![953.0, 9502.0],
                rates: vec![1.17, 0.91],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 1566.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_slo_draft_engine();
        let item = new_batched_slo_draft("code-269", "name-160");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
