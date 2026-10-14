use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global timeline proposal calculation.
pub trait GlobalTimelineProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalTimelineProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalTimelineProposalAdjuster {
    pub percent: f64,
}

impl GlobalTimelineProposalModifier for PercentGlobalTimelineProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalTimelineProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalTimelineProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalTimelineProposalModifier for TieredGlobalTimelineProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalTimelineProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTimelineProposalRateOffsetAdjuster {
    pub weight: f64,
}

impl GlobalTimelineProposalModifier for GlobalTimelineProposalRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalTimelineProposal, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalTimelineProposalAdjuster {
    pub cap: f64,
}

impl GlobalTimelineProposalModifier for CapGlobalTimelineProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalTimelineProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the global timeline proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalTimelineProposalEngine {
    pub steps: Vec<Rc<dyn GlobalTimelineProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalTimelineProposalEngine {
    pub fn run(&mut self, item: &GlobalTimelineProposal) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalTimelineProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalTimelineProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_timeline_proposal_engine() -> GlobalTimelineProposalEngine {
    GlobalTimelineProposalEngine {
        steps: vec![
            Rc::new(PercentGlobalTimelineProposalAdjuster { percent: -23.6 }) as Rc<dyn GlobalTimelineProposalModifier>,
            Rc::new(TieredGlobalTimelineProposalAdjuster {
                bounds: vec![4477.0, 5385.0],
                rates: vec![0.9, 0.91],
            }),
            Rc::new(GlobalTimelineProposalRateOffsetAdjuster { weight: 0.55 }),
            Rc::new(CapGlobalTimelineProposalAdjuster { cap: 26515.0 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 51913.1,
    }
}
