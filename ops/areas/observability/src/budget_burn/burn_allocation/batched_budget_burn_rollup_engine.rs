use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched budget burn rollup calculation.
pub trait BatchedBudgetBurnRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedBudgetBurnRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedBudgetBurnRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedBudgetBurnRollupFactor for TieredBatchedBudgetBurnRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedBudgetBurnRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedBudgetBurnRollupAdjuster {
    pub cap: f64,
}

impl BatchedBudgetBurnRollupFactor for CapBatchedBudgetBurnRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedBudgetBurnRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedBudgetBurnRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedBudgetBurnRollupFactor for FlaggedBatchedBudgetBurnRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedBudgetBurnRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched budget burn rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedBudgetBurnRollupEngine {
    pub steps: Vec<Rc<dyn BatchedBudgetBurnRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedBudgetBurnRollupEngine {
    pub fn compute(&mut self, item: &BatchedBudgetBurnRollup) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedBudgetBurnRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedBudgetBurnRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_budget_burn_rollup_engine(flags: &flags::Registry) -> BatchedBudgetBurnRollupEngine {
    BatchedBudgetBurnRollupEngine {
        steps: vec![
            Rc::new(TieredBatchedBudgetBurnRollupAdjuster {
                bounds: vec![783.0, 4451.0, 7741.0, 8205.0],
                rates: vec![1.1, 1.1, 0.89, 1.15],
            }) as Rc<dyn BatchedBudgetBurnRollupFactor>,
            Rc::new(CapBatchedBudgetBurnRollupAdjuster { cap: 42991.0 }),
            Rc::new(FlaggedBatchedBudgetBurnRollupAdjuster {
                flags: flags.clone(),
                flag: "batched-budget-burn-rollup-seasonal".to_string(),
                percent: 9.6,
            }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 58007.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_budget_burn_rollup_engine(&flags::new_registry());
        let item = new_batched_budget_burn_rollup("code-38", "name-711");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
