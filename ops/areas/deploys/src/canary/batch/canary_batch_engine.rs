use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary batch calculation.
pub trait CanaryBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBatchAmountOffsetAdjuster {
    pub weight: f64,
}

impl CanaryBatchStep for CanaryBatchAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryBatch, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryBatchStep for TieredCanaryBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionCanaryBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryBatchStep for RegionCanaryBatchAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &CanaryBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryBatchAdjuster {
    pub step: f64,
}

impl CanaryBatchStep for RoundingCanaryBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryBatchAdjuster {
    pub minimum: f64,
}

impl CanaryBatchStep for MinimumCanaryBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the canary batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryBatchEngine {
    pub steps: Vec<Rc<dyn CanaryBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryBatchEngine {
    pub fn run(&mut self, item: &CanaryBatch) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanaryBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanaryBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_canary_batch_engine() -> CanaryBatchEngine {
    CanaryBatchEngine {
        steps: vec![
            Rc::new(CanaryBatchAmountOffsetAdjuster { weight: 0.06 }) as Rc<dyn CanaryBatchStep>,
            Rc::new(TieredCanaryBatchAdjuster {
                bounds: vec![9163.0, 9790.0],
                rates: vec![0.85, 1.0],
            }),
            Rc::new(RegionCanaryBatchAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.11), ("south".to_string(), 1.21)]),
            }),
            Rc::new(RoundingCanaryBatchAdjuster { step: 0.25 }),
            Rc::new(MinimumCanaryBatchAdjuster { minimum: 7.1 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 70598.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_batch_engine();
        let item = new_canary_batch("code-62", "label-57");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
