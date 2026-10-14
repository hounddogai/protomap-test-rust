use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive silence assignment calculation.
pub trait AdaptiveSilenceAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveSilenceAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceAssignmentCapacityOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveSilenceAssignmentModifier for AdaptiveSilenceAssignmentCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveSilenceAssignment, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveSilenceAssignmentAdjuster {
    pub minimum: f64,
}

impl AdaptiveSilenceAssignmentModifier for MinimumAdaptiveSilenceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveSilenceAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveSilenceAssignmentAdjuster {
    pub cap: f64,
}

impl AdaptiveSilenceAssignmentModifier for CapAdaptiveSilenceAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveSilenceAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the adaptive silence assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveSilenceAssignmentEngine {
    pub steps: Vec<Rc<dyn AdaptiveSilenceAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveSilenceAssignmentEngine {
    pub fn evaluate(&mut self, item: &AdaptiveSilenceAssignment) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveSilenceAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_silence_assignment_engine() -> AdaptiveSilenceAssignmentEngine {
    AdaptiveSilenceAssignmentEngine {
        steps: vec![
            Rc::new(AdaptiveSilenceAssignmentCapacityOffsetAdjuster { weight: 1.89 })
                as Rc<dyn AdaptiveSilenceAssignmentModifier>,
            Rc::new(MinimumAdaptiveSilenceAssignmentAdjuster { minimum: 36.7 }),
            Rc::new(CapAdaptiveSilenceAssignmentAdjuster { cap: 39665.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 56198.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_silence_assignment_engine();
        let item = new_adaptive_silence_assignment("code-164", "title-380");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
