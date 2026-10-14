use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global segment assignment calculation.
pub trait GlobalSegmentAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSegmentAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalSegmentAssignmentAdjuster {
    pub percent: f64,
}

impl GlobalSegmentAssignmentStep for PercentGlobalSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalSegmentAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalSegmentAssignmentAdjuster {
    pub minimum: f64,
}

impl GlobalSegmentAssignmentStep for MinimumGlobalSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalSegmentAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalSegmentAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalSegmentAssignmentStep for FlaggedGlobalSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalSegmentAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global segment assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSegmentAssignmentEngine {
    pub steps: Vec<Rc<dyn GlobalSegmentAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSegmentAssignmentEngine {
    pub fn evaluate(&mut self, item: &GlobalSegmentAssignment) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalSegmentAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_segment_assignment_engine(flags: &flags::Registry) -> GlobalSegmentAssignmentEngine {
    GlobalSegmentAssignmentEngine {
        steps: vec![
            Rc::new(PercentGlobalSegmentAssignmentAdjuster { percent: -13.5 }) as Rc<dyn GlobalSegmentAssignmentStep>,
            Rc::new(MinimumGlobalSegmentAssignmentAdjuster { minimum: 40.4 }),
            Rc::new(FlaggedGlobalSegmentAssignmentAdjuster {
                flags: flags.clone(),
                flag: "global-segment-assignment-batched".to_string(),
                percent: -9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 68679.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_segment_assignment_engine(&flags::new_registry());
        let item = new_global_segment_assignment("code-3", "name-761");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
