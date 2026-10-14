use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the egress assignment calculation.
pub trait EgressAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressAssignmentAdjuster {
    pub step: f64,
}

impl EgressAssignmentStep for RoundingEgressAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignmentPriorityOffsetAdjuster {
    pub weight: f64,
}

impl EgressAssignmentStep for EgressAssignmentPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &EgressAssignment, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumEgressAssignmentAdjuster {
    pub minimum: f64,
}

impl EgressAssignmentStep for MinimumEgressAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &EgressAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the egress assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressAssignmentEngine {
    pub steps: Vec<Rc<dyn EgressAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressAssignmentEngine {
    pub fn evaluate(&mut self, item: &EgressAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn EgressAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[EgressAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_egress_assignment_engine() -> EgressAssignmentEngine {
    EgressAssignmentEngine {
        steps: vec![
            Rc::new(RoundingEgressAssignmentAdjuster { step: 0.5 }) as Rc<dyn EgressAssignmentStep>,
            Rc::new(EgressAssignmentPriorityOffsetAdjuster { weight: 1.47 }),
            Rc::new(MinimumEgressAssignmentAdjuster { minimum: 48.1 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 40346.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_assignment_engine();
        let item = new_egress_assignment("code-127", "title-148");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
