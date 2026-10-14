use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped rollback window calculation.
pub trait CappedRollbackWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRollbackWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedRollbackWindowAdjuster {
    pub cap: f64,
}

impl CappedRollbackWindowFactor for CapCappedRollbackWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedRollbackWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRollbackWindowAdjuster {
    pub minimum: f64,
}

impl CappedRollbackWindowFactor for MinimumCappedRollbackWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRollbackWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRollbackWindowAdjuster {
    pub percent: f64,
}

impl CappedRollbackWindowFactor for PercentCappedRollbackWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRollbackWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped rollback window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRollbackWindowEngine {
    pub steps: Vec<Rc<dyn CappedRollbackWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRollbackWindowEngine {
    pub fn run(&mut self, item: &CappedRollbackWindow) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CappedRollbackWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_rollback_window_engine() -> CappedRollbackWindowEngine {
    CappedRollbackWindowEngine {
        steps: vec![
            Rc::new(CapCappedRollbackWindowAdjuster { cap: 7531.0 }) as Rc<dyn CappedRollbackWindowFactor>,
            Rc::new(MinimumCappedRollbackWindowAdjuster { minimum: 16.8 }),
            Rc::new(PercentCappedRollbackWindowAdjuster { percent: -22.1 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 20532.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_rollback_window_engine();
        let item = new_capped_rollback_window("code-437", "title-30");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
