use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched failover summary calculation.
pub trait BatchedFailoverSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedFailoverSummaryAdjuster {
    pub cap: f64,
}

impl BatchedFailoverSummaryStep for CapBatchedFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleBatchedFailoverSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedFailoverSummaryStep for TitleBatchedFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedFailoverSummaryAdjuster {
    pub percent: f64,
}

impl BatchedFailoverSummaryStep for PercentBatchedFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedFailoverSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedFailoverSummaryStep for TieredBatchedFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedFailoverSummaryAdjuster {
    pub step: f64,
}

impl BatchedFailoverSummaryStep for RoundingBatchedFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFailoverSummaryThresholdOffsetAdjuster {
    pub weight: f64,
}

impl BatchedFailoverSummaryStep for BatchedFailoverSummaryThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedFailoverSummary, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

/// Runs the batched failover summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedFailoverSummaryEngine {
    pub steps: Vec<Rc<dyn BatchedFailoverSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedFailoverSummaryEngine {
    pub fn compute(&mut self, item: &BatchedFailoverSummary) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedFailoverSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedFailoverSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_failover_summary_engine() -> BatchedFailoverSummaryEngine {
    BatchedFailoverSummaryEngine {
        steps: vec![
            Rc::new(CapBatchedFailoverSummaryAdjuster { cap: 43572.0 }) as Rc<dyn BatchedFailoverSummaryStep>,
            Rc::new(TitleBatchedFailoverSummaryAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.85)]),
            }),
            Rc::new(PercentBatchedFailoverSummaryAdjuster { percent: -13.0 }),
            Rc::new(TieredBatchedFailoverSummaryAdjuster {
                bounds: vec![612.0, 5338.0, 5985.0, 6176.0],
                rates: vec![0.94, 0.87, 0.97, 0.94],
            }),
            Rc::new(RoundingBatchedFailoverSummaryAdjuster { step: 0.5 }),
            Rc::new(BatchedFailoverSummaryThresholdOffsetAdjuster { weight: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 86979.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_failover_summary_engine();
        let item = new_batched_failover_summary("code-730", "title-394");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
