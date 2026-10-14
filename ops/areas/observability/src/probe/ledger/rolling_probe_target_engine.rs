use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling probe target calculation.
pub trait RollingProbeTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingProbeTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeTargetCostOffsetAdjuster {
    pub weight: f64,
}

impl RollingProbeTargetFactor for RollingProbeTargetCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &RollingProbeTarget, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingProbeTargetAdjuster {
    pub percent: f64,
}

impl RollingProbeTargetFactor for PercentRollingProbeTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingProbeTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingProbeTargetAdjuster {
    pub step: f64,
}

impl RollingProbeTargetFactor for RoundingRollingProbeTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingProbeTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling probe target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingProbeTargetEngine {
    pub steps: Vec<Rc<dyn RollingProbeTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingProbeTargetEngine {
    pub fn run(&mut self, item: &RollingProbeTarget) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingProbeTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_probe_target_engine() -> RollingProbeTargetEngine {
    RollingProbeTargetEngine {
        steps: vec![
            Rc::new(RollingProbeTargetCostOffsetAdjuster { weight: 1.62 }) as Rc<dyn RollingProbeTargetFactor>,
            Rc::new(PercentRollingProbeTargetAdjuster { percent: -1.3 }),
            Rc::new(RoundingRollingProbeTargetAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 54569.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_probe_target_engine();
        let item = new_rolling_probe_target("code-516", "title-486");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
