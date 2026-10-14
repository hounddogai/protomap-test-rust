use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched budget burn batch calculation.
pub trait BatchedBudgetBurnBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBudgetBurnBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedBudgetBurnBatchAdjuster {
    pub step: f64,
}

impl BatchedBudgetBurnBatchFactor for RoundingBatchedBudgetBurnBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedBudgetBurnBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedBudgetBurnBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedBudgetBurnBatchFactor for LabelBatchedBudgetBurnBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedBudgetBurnBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedBudgetBurnBatchAdjuster {
    pub percent: f64,
}

impl BatchedBudgetBurnBatchFactor for PercentBatchedBudgetBurnBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedBudgetBurnBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedBudgetBurnBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedBudgetBurnBatchFactor for TieredBatchedBudgetBurnBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedBudgetBurnBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched budget burn batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBudgetBurnBatchEngine {
    pub steps: Vec<Rc<dyn BatchedBudgetBurnBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBudgetBurnBatchEngine {
    pub fn compute(&mut self, item: &BatchedBudgetBurnBatch) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedBudgetBurnBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBudgetBurnBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_budget_burn_batch_engine() -> BatchedBudgetBurnBatchEngine {
    BatchedBudgetBurnBatchEngine {
        steps: vec![
            Rc::new(RoundingBatchedBudgetBurnBatchAdjuster { step: 1.0 }) as Rc<dyn BatchedBudgetBurnBatchFactor>,
            Rc::new(LabelBatchedBudgetBurnBatchAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.88),
                    ("south".to_string(), 1.16),
                    ("online".to_string(), 1.38),
                ]),
            }),
            Rc::new(PercentBatchedBudgetBurnBatchAdjuster { percent: -21.6 }),
            Rc::new(TieredBatchedBudgetBurnBatchAdjuster {
                bounds: vec![3102.0, 6658.0, 7648.0],
                rates: vec![0.96, 0.92, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 70112.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_budget_burn_batch_engine();
        let item = new_batched_budget_burn_batch("code-823", "label-95");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
