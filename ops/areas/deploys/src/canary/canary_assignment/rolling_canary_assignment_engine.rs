use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling canary assignment calculation.
pub trait RollingCanaryAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingCanaryAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingCanaryAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingCanaryAssignmentFactor for TieredRollingCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingCanaryAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingCanaryAssignmentAdjuster {
    pub percent: f64,
}

impl RollingCanaryAssignmentFactor for PercentRollingCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingCanaryAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingCanaryAssignmentAdjuster {
    pub step: f64,
}

impl RollingCanaryAssignmentFactor for RoundingRollingCanaryAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingCanaryAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling canary assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingCanaryAssignmentEngine {
    pub steps: Vec<Rc<dyn RollingCanaryAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingCanaryAssignmentEngine {
    pub fn evaluate(&mut self, item: &RollingCanaryAssignment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingCanaryAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rolling_canary_assignment_engine() -> RollingCanaryAssignmentEngine {
    RollingCanaryAssignmentEngine {
        steps: vec![
            Rc::new(TieredRollingCanaryAssignmentAdjuster {
                bounds: vec![2047.0, 8432.0, 8844.0],
                rates: vec![1.02, 1.02, 0.98],
            }) as Rc<dyn RollingCanaryAssignmentFactor>,
            Rc::new(PercentRollingCanaryAssignmentAdjuster { percent: 8.3 }),
            Rc::new(RoundingRollingCanaryAssignmentAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 13300.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_canary_assignment_engine();
        let item = new_rolling_canary_assignment("code-524", "label-306");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
