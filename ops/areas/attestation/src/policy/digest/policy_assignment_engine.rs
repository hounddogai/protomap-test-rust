use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the policy assignment calculation.
pub trait PolicyAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyAssignmentAdjuster {
    pub percent: f64,
}

impl PolicyAssignmentModifier for PercentPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyAssignmentAdjuster {
    pub step: f64,
}

impl PolicyAssignmentModifier for RoundingPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyAssignmentAdjuster {
    pub cap: f64,
}

impl PolicyAssignmentModifier for CapPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyAssignmentModifier for TieredPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyAssignmentAdjuster {
    pub minimum: f64,
}

impl PolicyAssignmentModifier for MinimumPolicyAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the policy assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyAssignmentEngine {
    pub steps: Vec<Rc<dyn PolicyAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyAssignmentEngine {
    pub fn compute(&mut self, item: &PolicyAssignment) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_assignment_engine() -> PolicyAssignmentEngine {
    PolicyAssignmentEngine {
        steps: vec![
            Rc::new(PercentPolicyAssignmentAdjuster { percent: 22.7 }) as Rc<dyn PolicyAssignmentModifier>,
            Rc::new(RoundingPolicyAssignmentAdjuster { step: 0.5 }),
            Rc::new(CapPolicyAssignmentAdjuster { cap: 20353.0 }),
            Rc::new(TieredPolicyAssignmentAdjuster {
                bounds: vec![1761.0, 1824.0, 9017.0],
                rates: vec![1.09, 1.08, 0.93],
            }),
            Rc::new(MinimumPolicyAssignmentAdjuster { minimum: 4.7 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 21876.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_assignment_engine();
        let item = new_policy_assignment("code-906", "title-453");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
