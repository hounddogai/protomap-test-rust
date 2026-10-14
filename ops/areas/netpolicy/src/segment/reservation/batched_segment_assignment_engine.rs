use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched segment assignment calculation.
pub trait BatchedSegmentAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSegmentAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSegmentAssignmentAdjuster {
    pub cap: f64,
}

impl BatchedSegmentAssignmentAdjuster for CapBatchedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSegmentAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedSegmentAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedSegmentAssignmentAdjuster for FlaggedBatchedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedSegmentAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedSegmentAssignmentAdjuster {
    pub minimum: f64,
}

impl BatchedSegmentAssignmentAdjuster for MinimumBatchedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedSegmentAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedSegmentAssignmentAdjuster {
    pub step: f64,
}

impl BatchedSegmentAssignmentAdjuster for RoundingBatchedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedSegmentAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameBatchedSegmentAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSegmentAssignmentAdjuster for NameBatchedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &BatchedSegmentAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched segment assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSegmentAssignmentEngine {
    pub steps: Vec<Rc<dyn BatchedSegmentAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSegmentAssignmentEngine {
    pub fn run(&mut self, item: &BatchedSegmentAssignment) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSegmentAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_segment_assignment_engine(flags: &flags::Registry) -> BatchedSegmentAssignmentEngine {
    BatchedSegmentAssignmentEngine {
        steps: vec![
            Rc::new(CapBatchedSegmentAssignmentAdjuster { cap: 23205.0 }) as Rc<dyn BatchedSegmentAssignmentAdjuster>,
            Rc::new(FlaggedBatchedSegmentAssignmentAdjuster {
                flags: flags.clone(),
                flag: "batched-segment-assignment-manual".to_string(),
                percent: -3.8,
            }),
            Rc::new(MinimumBatchedSegmentAssignmentAdjuster { minimum: 46.8 }),
            Rc::new(RoundingBatchedSegmentAssignmentAdjuster { step: 0.1 }),
            Rc::new(NameBatchedSegmentAssignmentAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.23)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 36469.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_segment_assignment_engine(&flags::new_registry());
        let item = new_batched_segment_assignment("code-534", "name-575");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
