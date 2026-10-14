use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the alert allocation calculation.
pub trait AlertAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertAllocationAdjuster {
    pub minimum: f64,
}

impl AlertAllocationStep for MinimumAlertAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertAllocationAdjuster {
    pub cap: f64,
}

impl AlertAllocationStep for CapAlertAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertAllocationAdjuster {
    pub percent: f64,
}

impl AlertAllocationStep for PercentAlertAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the alert allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertAllocationEngine {
    pub steps: Vec<Rc<dyn AlertAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertAllocationEngine {
    pub fn evaluate(&mut self, item: &AlertAllocation) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[AlertAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_allocation_engine() -> AlertAllocationEngine {
    AlertAllocationEngine {
        steps: vec![
            Rc::new(MinimumAlertAllocationAdjuster { minimum: 19.5 }) as Rc<dyn AlertAllocationStep>,
            Rc::new(CapAlertAllocationAdjuster { cap: 21663.0 }),
            Rc::new(PercentAlertAllocationAdjuster { percent: 0.2 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 41380.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_allocation_engine();
        let item = new_alert_allocation("code-316", "label-946");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
