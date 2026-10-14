use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the budget burn batch calculation.
pub trait BudgetBurnBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnBatchAdjuster {
    pub minimum: f64,
}

impl BudgetBurnBatchFactor for MinimumBudgetBurnBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnBatchAdjuster {
    pub percent: f64,
}

impl BudgetBurnBatchFactor for PercentBudgetBurnBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatchMarginOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnBatchFactor for BudgetBurnBatchMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnBatch, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the budget burn batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnBatchEngine {
    pub steps: Vec<Rc<dyn BudgetBurnBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnBatchEngine {
    pub fn price(&mut self, item: &BudgetBurnBatch) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnBatchFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_batch_engine() -> BudgetBurnBatchEngine {
    BudgetBurnBatchEngine {
        steps: vec![
            Rc::new(MinimumBudgetBurnBatchAdjuster { minimum: 8.2 }) as Rc<dyn BudgetBurnBatchFactor>,
            Rc::new(PercentBudgetBurnBatchAdjuster { percent: -11.9 }),
            Rc::new(BudgetBurnBatchMarginOffsetAdjuster { weight: 1.44 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 45404.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_batch_engine();
        let item = new_budget_burn_batch("code-146", "label-582");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
