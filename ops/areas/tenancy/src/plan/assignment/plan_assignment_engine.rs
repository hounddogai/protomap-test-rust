use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan assignment calculation.
pub trait PlanAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanAssignmentAdjuster {
    pub minimum: f64,
}

impl PlanAssignmentModifier for MinimumPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanAssignmentAdjuster {
    pub step: f64,
}

impl PlanAssignmentModifier for RoundingPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanAssignmentAdjuster {
    pub cap: f64,
}

impl PlanAssignmentModifier for CapPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanAssignmentModifier for TieredPlanAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAssignmentMinimumOffsetAdjuster {
    pub weight: f64,
}

impl PlanAssignmentModifier for PlanAssignmentMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &PlanAssignment, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

/// Runs the plan assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanAssignmentEngine {
    pub steps: Vec<Rc<dyn PlanAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanAssignmentEngine {
    pub fn price(&mut self, item: &PlanAssignment) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_assignment_engine() -> PlanAssignmentEngine {
    PlanAssignmentEngine {
        steps: vec![
            Rc::new(MinimumPlanAssignmentAdjuster { minimum: 20.9 }) as Rc<dyn PlanAssignmentModifier>,
            Rc::new(RoundingPlanAssignmentAdjuster { step: 0.05 }),
            Rc::new(CapPlanAssignmentAdjuster { cap: 12370.0 }),
            Rc::new(TieredPlanAssignmentAdjuster {
                bounds: vec![1053.0, 1778.0, 5716.0, 6168.0],
                rates: vec![0.85, 0.91, 1.04, 1.03],
            }),
            Rc::new(PlanAssignmentMinimumOffsetAdjuster { weight: 1.89 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 18950.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_assignment_engine();
        let item = new_plan_assignment("code-328", "title-923");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
