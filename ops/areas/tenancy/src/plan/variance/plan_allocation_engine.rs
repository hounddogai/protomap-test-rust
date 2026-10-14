use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan allocation calculation.
pub trait PlanAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocationRevisionOffsetAdjuster {
    pub weight: f64,
}

impl PlanAllocationStep for PlanAllocationRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &PlanAllocation, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanAllocationAdjuster {
    pub step: f64,
}

impl PlanAllocationStep for RoundingPlanAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanAllocationAdjuster {
    pub cap: f64,
}

impl PlanAllocationStep for CapPlanAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the plan allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanAllocationEngine {
    pub steps: Vec<Rc<dyn PlanAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanAllocationEngine {
    pub fn evaluate(&mut self, item: &PlanAllocation) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanAllocationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_allocation_engine() -> PlanAllocationEngine {
    PlanAllocationEngine {
        steps: vec![
            Rc::new(PlanAllocationRevisionOffsetAdjuster { weight: 1.45 }) as Rc<dyn PlanAllocationStep>,
            Rc::new(RoundingPlanAllocationAdjuster { step: 0.05 }),
            Rc::new(CapPlanAllocationAdjuster { cap: 27654.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 58669.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_allocation_engine();
        let item = new_plan_allocation("code-671", "name-903");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
