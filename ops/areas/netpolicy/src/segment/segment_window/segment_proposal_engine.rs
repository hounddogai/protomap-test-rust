use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the segment proposal calculation.
pub trait SegmentProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentProposalAdjuster {
    pub cap: f64,
}

impl SegmentProposalAdjuster for CapSegmentProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentProposalAdjuster {
    pub percent: f64,
}

impl SegmentProposalAdjuster for PercentSegmentProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentProposalAdjuster {
    pub minimum: f64,
}

impl SegmentProposalAdjuster for MinimumSegmentProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentProposalBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl SegmentProposalAdjuster for SegmentProposalBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentProposal, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the segment proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentProposalEngine {
    pub steps: Vec<Rc<dyn SegmentProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentProposalEngine {
    pub fn compute(&mut self, item: &SegmentProposal) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_proposal_engine() -> SegmentProposalEngine {
    SegmentProposalEngine {
        steps: vec![
            Rc::new(CapSegmentProposalAdjuster { cap: 47304.0 }) as Rc<dyn SegmentProposalAdjuster>,
            Rc::new(PercentSegmentProposalAdjuster { percent: 3.5 }),
            Rc::new(MinimumSegmentProposalAdjuster { minimum: 44.8 }),
            Rc::new(SegmentProposalBatchSizeOffsetAdjuster { weight: 0.34 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 81174.3,
    }
}
