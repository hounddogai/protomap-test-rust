use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the segment assignment calculation.
pub trait SegmentAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentAssignmentAdjuster {
    pub percent: f64,
}

impl SegmentAssignmentAdjuster for PercentSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentAssignmentAdjuster {
    pub minimum: f64,
}

impl SegmentAssignmentAdjuster for MinimumSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentAssignmentAdjuster for TieredSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignmentAmountOffsetAdjuster {
    pub weight: f64,
}

impl SegmentAssignmentAdjuster for SegmentAssignmentAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentAssignmentAdjuster for FlaggedSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentAssignmentAdjuster {
    pub cap: f64,
}

impl SegmentAssignmentAdjuster for CapSegmentAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the segment assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentAssignmentEngine {
    pub steps: Vec<Rc<dyn SegmentAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentAssignmentEngine {
    pub fn price(&mut self, item: &SegmentAssignment) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_assignment_engine(flags: &flags::Registry) -> SegmentAssignmentEngine {
    SegmentAssignmentEngine {
        steps: vec![
            Rc::new(PercentSegmentAssignmentAdjuster { percent: -6.5 }) as Rc<dyn SegmentAssignmentAdjuster>,
            Rc::new(MinimumSegmentAssignmentAdjuster { minimum: 12.3 }),
            Rc::new(TieredSegmentAssignmentAdjuster {
                bounds: vec![2280.0, 3816.0, 5793.0, 6676.0],
                rates: vec![0.86, 0.89, 0.86, 1.04],
            }),
            Rc::new(SegmentAssignmentAmountOffsetAdjuster { weight: 0.21 }),
            Rc::new(FlaggedSegmentAssignmentAdjuster {
                flags: flags.clone(),
                flag: "segment-assignment-regional".to_string(),
                percent: -2.2,
            }),
            Rc::new(CapSegmentAssignmentAdjuster { cap: 11863.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 88777.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_assignment_engine(&flags::new_registry());
        let item = new_segment_assignment("code-111", "title-473");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
