use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback assignment calculation.
pub trait RollbackAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackAssignmentAdjuster {
    pub percent: f64,
}

impl RollbackAssignmentStep for PercentRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackAssignmentAdjuster {
    pub step: f64,
}

impl RollbackAssignmentStep for RoundingRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAssignmentRateOffsetAdjuster {
    pub weight: f64,
}

impl RollbackAssignmentStep for RollbackAssignmentRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackAssignment, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeRollbackAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackAssignmentStep for ScopeRollbackAssignmentAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &RollbackAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rollback assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackAssignmentEngine {
    pub steps: Vec<Rc<dyn RollbackAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackAssignmentEngine {
    pub fn evaluate(&mut self, item: &RollbackAssignment) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollbackAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollbackAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rollback_assignment_engine() -> RollbackAssignmentEngine {
    RollbackAssignmentEngine {
        steps: vec![
            Rc::new(PercentRollbackAssignmentAdjuster { percent: -20.6 }) as Rc<dyn RollbackAssignmentStep>,
            Rc::new(RoundingRollbackAssignmentAdjuster { step: 0.1 }),
            Rc::new(RollbackAssignmentRateOffsetAdjuster { weight: 1.95 }),
            Rc::new(ScopeRollbackAssignmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.39)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 44154.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_assignment_engine();
        let item = new_rollback_assignment("code-297", "label-599");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
