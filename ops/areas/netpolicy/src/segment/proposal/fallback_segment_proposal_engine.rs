use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback segment proposal calculation.
pub trait FallbackSegmentProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSegmentProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackSegmentProposalAdjuster {
    pub minimum: f64,
}

impl FallbackSegmentProposalStep for MinimumFallbackSegmentProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackSegmentProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackSegmentProposalAdjuster {
    pub step: f64,
}

impl FallbackSegmentProposalStep for RoundingFallbackSegmentProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackSegmentProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackSegmentProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackSegmentProposalStep for FlaggedFallbackSegmentProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackSegmentProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFallbackSegmentProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackSegmentProposalStep for TitleFallbackSegmentProposalAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FallbackSegmentProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback segment proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSegmentProposalEngine {
    pub steps: Vec<Rc<dyn FallbackSegmentProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSegmentProposalEngine {
    pub fn price(&mut self, item: &FallbackSegmentProposal) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackSegmentProposalStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSegmentProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_segment_proposal_engine(flags: &flags::Registry) -> FallbackSegmentProposalEngine {
    FallbackSegmentProposalEngine {
        steps: vec![
            Rc::new(MinimumFallbackSegmentProposalAdjuster { minimum: 35.7 }) as Rc<dyn FallbackSegmentProposalStep>,
            Rc::new(RoundingFallbackSegmentProposalAdjuster { step: 0.5 }),
            Rc::new(FlaggedFallbackSegmentProposalAdjuster {
                flags: flags.clone(),
                flag: "fallback-segment-proposal-scheduled".to_string(),
                percent: 1.8,
            }),
            Rc::new(TitleFallbackSegmentProposalAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.8)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 96497.7,
    }
}
