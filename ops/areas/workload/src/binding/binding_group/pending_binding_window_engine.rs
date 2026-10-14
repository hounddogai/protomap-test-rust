use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending binding window calculation.
pub trait PendingBindingWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingBindingWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingBindingWindowAdjuster {
    pub minimum: f64,
}

impl PendingBindingWindowAdjuster for MinimumPendingBindingWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingBindingWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingWindowRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl PendingBindingWindowAdjuster for PendingBindingWindowRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &PendingBindingWindow, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingBindingWindowAdjuster {
    pub cap: f64,
}

impl PendingBindingWindowAdjuster for CapPendingBindingWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingBindingWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending binding window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingBindingWindowEngine {
    pub steps: Vec<Rc<dyn PendingBindingWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingBindingWindowEngine {
    pub fn evaluate(&mut self, item: &PendingBindingWindow) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PendingBindingWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_binding_window_engine() -> PendingBindingWindowEngine {
    PendingBindingWindowEngine {
        steps: vec![
            Rc::new(MinimumPendingBindingWindowAdjuster { minimum: 22.1 }) as Rc<dyn PendingBindingWindowAdjuster>,
            Rc::new(PendingBindingWindowRotationDaysOffsetAdjuster { weight: 0.27 }),
            Rc::new(CapPendingBindingWindowAdjuster { cap: 14296.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 44182.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_binding_window_engine();
        let item = new_pending_binding_window("code-836", "name-770");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
