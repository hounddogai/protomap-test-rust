use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the release proposal calculation.
pub trait ReleaseProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleaseProposalAdjuster {
    pub minimum: f64,
}

impl ReleaseProposalAdjuster for MinimumReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseProposalAdjuster for FlaggedReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposalAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseProposalAdjuster for ReleaseProposalAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReleaseProposalAdjuster {
    pub step: f64,
}

impl ReleaseProposalAdjuster for RoundingReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseProposalAdjuster for TieredReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseProposalAdjuster {
    pub percent: f64,
}

impl ReleaseProposalAdjuster for PercentReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseProposalEngine {
    pub steps: Vec<Rc<dyn ReleaseProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseProposalEngine {
    pub fn compute(&mut self, item: &ReleaseProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_proposal_engine(flags: &flags::Registry) -> ReleaseProposalEngine {
    ReleaseProposalEngine {
        steps: vec![
            Rc::new(MinimumReleaseProposalAdjuster { minimum: 48.7 }) as Rc<dyn ReleaseProposalAdjuster>,
            Rc::new(FlaggedReleaseProposalAdjuster {
                flags: flags.clone(),
                flag: "release-proposal-tiered".to_string(),
                percent: 7.4,
            }),
            Rc::new(ReleaseProposalAttemptsOffsetAdjuster { weight: 0.36 }),
            Rc::new(RoundingReleaseProposalAdjuster { step: 0.1 }),
            Rc::new(TieredReleaseProposalAdjuster {
                bounds: vec![1454.0, 3725.0, 7212.0, 9590.0],
                rates: vec![0.94, 0.95, 0.97, 1.17],
            }),
            Rc::new(PercentReleaseProposalAdjuster { percent: -12.6 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 83973.2,
    }
}
