use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global alert cycle calculation.
pub trait GlobalAlertCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalAlertCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalAlertCycleAdjuster {
    pub minimum: f64,
}

impl GlobalAlertCycleStep for MinimumGlobalAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalAlertCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalAlertCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalAlertCycleStep for TieredGlobalAlertCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalAlertCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAlertCycleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl GlobalAlertCycleStep for GlobalAlertCycleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalAlertCycle, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

/// Runs the global alert cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalAlertCycleEngine {
    pub steps: Vec<Rc<dyn GlobalAlertCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalAlertCycleEngine {
    pub fn evaluate(&mut self, item: &GlobalAlertCycle) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalAlertCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalAlertCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_alert_cycle_engine() -> GlobalAlertCycleEngine {
    GlobalAlertCycleEngine {
        steps: vec![
            Rc::new(MinimumGlobalAlertCycleAdjuster { minimum: 16.6 }) as Rc<dyn GlobalAlertCycleStep>,
            Rc::new(TieredGlobalAlertCycleAdjuster {
                bounds: vec![635.0, 5742.0],
                rates: vec![1.18, 1.03],
            }),
            Rc::new(GlobalAlertCycleQuantityOffsetAdjuster { weight: 1.97 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 76276.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_alert_cycle_engine();
        let item = new_global_alert_cycle("code-528", "name-836");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
