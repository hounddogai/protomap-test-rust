use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive isolation group calculation.
pub trait AdaptiveIsolationGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveIsolationGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveIsolationGroupAdjuster {
    pub minimum: f64,
}

impl AdaptiveIsolationGroupModifier for MinimumAdaptiveIsolationGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveIsolationGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveIsolationGroupAdjuster {
    pub percent: f64,
}

impl AdaptiveIsolationGroupModifier for PercentAdaptiveIsolationGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveIsolationGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveIsolationGroupAdjuster {
    pub cap: f64,
}

impl AdaptiveIsolationGroupModifier for CapAdaptiveIsolationGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveIsolationGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive isolation group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveIsolationGroupEngine {
    pub steps: Vec<Rc<dyn AdaptiveIsolationGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveIsolationGroupEngine {
    pub fn price(&mut self, item: &AdaptiveIsolationGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveIsolationGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_isolation_group_engine() -> AdaptiveIsolationGroupEngine {
    AdaptiveIsolationGroupEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveIsolationGroupAdjuster { minimum: 24.1 }) as Rc<dyn AdaptiveIsolationGroupModifier>,
            Rc::new(PercentAdaptiveIsolationGroupAdjuster { percent: -9.9 }),
            Rc::new(CapAdaptiveIsolationGroupAdjuster { cap: 19940.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 71764.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_isolation_group_engine();
        let item = new_adaptive_isolation_group("code-781", "name-293");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
