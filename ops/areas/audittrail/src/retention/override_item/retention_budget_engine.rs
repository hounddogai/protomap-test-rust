use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention budget calculation.
pub trait RetentionBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionBudgetAdjuster {
    pub cap: f64,
}

impl RetentionBudgetStep for CapRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionBudgetStep for TieredRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionBudgetAdjuster {
    pub step: f64,
}

impl RetentionBudgetStep for RoundingRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionBudgetAdjuster {
    pub percent: f64,
}

impl RetentionBudgetStep for PercentRetentionBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the retention budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionBudgetEngine {
    pub steps: Vec<Rc<dyn RetentionBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionBudgetEngine {
    pub fn compute(&mut self, item: &RetentionBudget) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_retention_budget_engine() -> RetentionBudgetEngine {
    RetentionBudgetEngine {
        steps: vec![
            Rc::new(CapRetentionBudgetAdjuster { cap: 23477.0 }) as Rc<dyn RetentionBudgetStep>,
            Rc::new(TieredRetentionBudgetAdjuster {
                bounds: vec![2932.0, 6551.0, 7128.0],
                rates: vec![0.89, 1.03, 0.92],
            }),
            Rc::new(RoundingRetentionBudgetAdjuster { step: 0.25 }),
            Rc::new(PercentRetentionBudgetAdjuster { percent: -23.9 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 46271.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_budget_engine();
        let item = new_retention_budget("code-530", "name-967");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
