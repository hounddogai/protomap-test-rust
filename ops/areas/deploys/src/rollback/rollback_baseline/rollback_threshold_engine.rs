use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rollback threshold calculation.
pub trait RollbackThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackThresholdAdjuster {
    pub minimum: f64,
}

impl RollbackThresholdStep for MinimumRollbackThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackThresholdAdjuster {
    pub cap: f64,
}

impl RollbackThresholdStep for CapRollbackThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackThresholdStep for TieredRollbackThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rollback threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackThresholdEngine {
    pub steps: Vec<Rc<dyn RollbackThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackThresholdEngine {
    pub fn evaluate(&mut self, item: &RollbackThreshold) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollback_threshold_engine() -> RollbackThresholdEngine {
    RollbackThresholdEngine {
        steps: vec![
            Rc::new(MinimumRollbackThresholdAdjuster { minimum: 35.8 }) as Rc<dyn RollbackThresholdStep>,
            Rc::new(CapRollbackThresholdAdjuster { cap: 17021.0 }),
            Rc::new(TieredRollbackThresholdAdjuster {
                bounds: vec![6663.0, 7012.0],
                rates: vec![0.87, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 61410.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_threshold_engine();
        let item = new_rollback_threshold("code-87", "label-98");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
