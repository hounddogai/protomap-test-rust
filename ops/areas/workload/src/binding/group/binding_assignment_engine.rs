use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding assignment calculation.
pub trait BindingAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingAssignmentAdjuster {
    pub minimum: f64,
}

impl BindingAssignmentStep for MinimumBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingAssignmentAdjuster {
    pub step: f64,
}

impl BindingAssignmentStep for RoundingBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingAssignmentAdjuster {
    pub percent: f64,
}

impl BindingAssignmentStep for PercentBindingAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingAssignmentVersionOffsetAdjuster {
    pub weight: f64,
}

impl BindingAssignmentStep for BindingAssignmentVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &BindingAssignment, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the binding assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingAssignmentEngine {
    pub steps: Vec<Rc<dyn BindingAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingAssignmentEngine {
    pub fn evaluate(&mut self, item: &BindingAssignment) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[BindingAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_assignment_engine() -> BindingAssignmentEngine {
    BindingAssignmentEngine {
        steps: vec![
            Rc::new(MinimumBindingAssignmentAdjuster { minimum: 49.8 }) as Rc<dyn BindingAssignmentStep>,
            Rc::new(RoundingBindingAssignmentAdjuster { step: 0.5 }),
            Rc::new(PercentBindingAssignmentAdjuster { percent: -5.7 }),
            Rc::new(BindingAssignmentVersionOffsetAdjuster { weight: 0.17 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 15568.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_assignment_engine();
        let item = new_binding_assignment("code-884", "title-612");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
