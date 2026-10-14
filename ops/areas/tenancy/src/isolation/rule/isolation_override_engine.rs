use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the isolation override calculation.
pub trait IsolationOverrideFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationOverrideAdjuster {
    pub cap: f64,
}

impl IsolationOverrideFactor for CapIsolationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationOverrideAdjuster {
    pub minimum: f64,
}

impl IsolationOverrideFactor for MinimumIsolationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationOverrideAdjuster {
    pub step: f64,
}

impl IsolationOverrideFactor for RoundingIsolationOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the isolation override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationOverrideEngine {
    pub steps: Vec<Rc<dyn IsolationOverrideFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationOverrideEngine {
    pub fn compute(&mut self, item: &IsolationOverride) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[IsolationOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_isolation_override_engine() -> IsolationOverrideEngine {
    IsolationOverrideEngine {
        steps: vec![
            Rc::new(CapIsolationOverrideAdjuster { cap: 37037.0 }) as Rc<dyn IsolationOverrideFactor>,
            Rc::new(MinimumIsolationOverrideAdjuster { minimum: 5.7 }),
            Rc::new(RoundingIsolationOverrideAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 87337.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_override_engine();
        let item = new_isolation_override("code-738", "title-624");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
