use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention cycle calculation.
pub trait RetentionCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionCycleAdjuster {
    pub minimum: f64,
}

impl RetentionCycleStep for MinimumRetentionCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionCycleAdjuster {
    pub step: f64,
}

impl RetentionCycleStep for RoundingRetentionCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionCycleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl RetentionCycleStep for RetentionCycleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionCycle, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionCycleStep for TieredRetentionCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionCycleAdjuster {
    pub cap: f64,
}

impl RetentionCycleStep for CapRetentionCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the retention cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionCycleEngine {
    pub steps: Vec<Rc<dyn RetentionCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionCycleEngine {
    pub fn evaluate(&mut self, item: &RetentionCycle) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_cycle_engine() -> RetentionCycleEngine {
    RetentionCycleEngine {
        steps: vec![
            Rc::new(MinimumRetentionCycleAdjuster { minimum: 39.6 }) as Rc<dyn RetentionCycleStep>,
            Rc::new(RoundingRetentionCycleAdjuster { step: 0.05 }),
            Rc::new(RetentionCycleMaximumOffsetAdjuster { weight: 0.72 }),
            Rc::new(TieredRetentionCycleAdjuster {
                bounds: vec![1437.0, 1560.0, 9281.0],
                rates: vec![0.86, 1.1, 1.15],
            }),
            Rc::new(CapRetentionCycleAdjuster { cap: 25421.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 96808.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_cycle_engine();
        let item = new_retention_cycle("code-549", "name-330");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
