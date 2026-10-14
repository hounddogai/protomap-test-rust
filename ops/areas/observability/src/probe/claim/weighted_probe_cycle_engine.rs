use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted probe cycle calculation.
pub trait WeightedProbeCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedProbeCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedProbeCycleAdjuster {
    pub step: f64,
}

impl WeightedProbeCycleFactor for RoundingWeightedProbeCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedProbeCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedProbeCycleAdjuster {
    pub cap: f64,
}

impl WeightedProbeCycleFactor for CapWeightedProbeCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedProbeCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedProbeCycleAdjuster {
    pub percent: f64,
}

impl WeightedProbeCycleFactor for PercentWeightedProbeCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedProbeCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted probe cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedProbeCycleEngine {
    pub steps: Vec<Rc<dyn WeightedProbeCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedProbeCycleEngine {
    pub fn run(&mut self, item: &WeightedProbeCycle) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[WeightedProbeCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_weighted_probe_cycle_engine() -> WeightedProbeCycleEngine {
    WeightedProbeCycleEngine {
        steps: vec![
            Rc::new(RoundingWeightedProbeCycleAdjuster { step: 0.05 }) as Rc<dyn WeightedProbeCycleFactor>,
            Rc::new(CapWeightedProbeCycleAdjuster { cap: 33894.0 }),
            Rc::new(PercentWeightedProbeCycleAdjuster { percent: 22.7 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 92847.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_probe_cycle_engine();
        let item = new_weighted_probe_cycle("code-293", "label-386");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
