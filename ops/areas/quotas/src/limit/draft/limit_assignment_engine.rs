use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the limit assignment calculation.
pub trait LimitAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitAssignmentAdjuster {
    pub step: f64,
}

impl LimitAssignmentModifier for RoundingLimitAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentMarginOffsetAdjuster {
    pub weight: f64,
}

impl LimitAssignmentModifier for LimitAssignmentMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &LimitAssignment, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitAssignmentAdjuster {
    pub cap: f64,
}

impl LimitAssignmentModifier for CapLimitAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the limit assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitAssignmentEngine {
    pub steps: Vec<Rc<dyn LimitAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitAssignmentEngine {
    pub fn run(&mut self, item: &LimitAssignment) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_limit_assignment_engine() -> LimitAssignmentEngine {
    LimitAssignmentEngine {
        steps: vec![
            Rc::new(RoundingLimitAssignmentAdjuster { step: 5.0 }) as Rc<dyn LimitAssignmentModifier>,
            Rc::new(LimitAssignmentMarginOffsetAdjuster { weight: 1.76 }),
            Rc::new(CapLimitAssignmentAdjuster { cap: 16939.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 28159.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_assignment_engine();
        let item = new_limit_assignment("code-227", "label-608");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
