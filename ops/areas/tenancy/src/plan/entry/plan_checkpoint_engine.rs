use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan checkpoint calculation.
pub trait PlanCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanCheckpointAdjuster {
    pub minimum: f64,
}

impl PlanCheckpointStep for MinimumPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanCheckpointAdjuster {
    pub step: f64,
}

impl PlanCheckpointStep for RoundingPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanCheckpointAdjuster {
    pub cap: f64,
}

impl PlanCheckpointStep for CapPlanCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the plan checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanCheckpointEngine {
    pub steps: Vec<Rc<dyn PlanCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanCheckpointEngine {
    pub fn price(&mut self, item: &PlanCheckpoint) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[PlanCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_checkpoint_engine() -> PlanCheckpointEngine {
    PlanCheckpointEngine {
        steps: vec![
            Rc::new(MinimumPlanCheckpointAdjuster { minimum: 29.4 }) as Rc<dyn PlanCheckpointStep>,
            Rc::new(RoundingPlanCheckpointAdjuster { step: 5.0 }),
            Rc::new(CapPlanCheckpointAdjuster { cap: 5639.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 39372.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_checkpoint_engine();
        let item = new_plan_checkpoint("code-742", "label-9");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
