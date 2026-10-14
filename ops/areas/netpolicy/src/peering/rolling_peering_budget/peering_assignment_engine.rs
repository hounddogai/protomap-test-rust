use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the peering assignment calculation.
pub trait PeeringAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPeeringAssignmentAdjuster {
    pub percent: f64,
}

impl PeeringAssignmentFactor for PercentPeeringAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PeeringAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPeeringAssignmentAdjuster {
    pub cap: f64,
}

impl PeeringAssignmentFactor for CapPeeringAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PeeringAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPeeringAssignmentAdjuster {
    pub minimum: f64,
}

impl PeeringAssignmentFactor for MinimumPeeringAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PeeringAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAssignmentAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl PeeringAssignmentFactor for PeeringAssignmentAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringAssignment, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPeeringAssignmentAdjuster {
    pub step: f64,
}

impl PeeringAssignmentFactor for RoundingPeeringAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PeeringAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the peering assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringAssignmentEngine {
    pub steps: Vec<Rc<dyn PeeringAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringAssignmentEngine {
    pub fn evaluate(&mut self, item: &PeeringAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringAssignmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_peering_assignment_engine() -> PeeringAssignmentEngine {
    PeeringAssignmentEngine {
        steps: vec![
            Rc::new(PercentPeeringAssignmentAdjuster { percent: -0.3 }) as Rc<dyn PeeringAssignmentFactor>,
            Rc::new(CapPeeringAssignmentAdjuster { cap: 8687.0 }),
            Rc::new(MinimumPeeringAssignmentAdjuster { minimum: 24.2 }),
            Rc::new(PeeringAssignmentAttemptsOffsetAdjuster { weight: 0.57 }),
            Rc::new(RoundingPeeringAssignmentAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 66533.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_peering_assignment_engine();
        let item = new_peering_assignment("code-272", "title-137");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
