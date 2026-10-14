use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending retention proposal calculation.
pub trait PendingRetentionProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingRetentionProposalAdjuster {
    pub step: f64,
}

impl PendingRetentionProposalAdjuster for RoundingPendingRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingRetentionProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRetentionProposalAdjuster for NamePendingRetentionProposalAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionProposalSequenceOffsetAdjuster {
    pub weight: f64,
}

impl PendingRetentionProposalAdjuster for PendingRetentionProposalSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRetentionProposalAdjuster {
    pub minimum: f64,
}

impl PendingRetentionProposalAdjuster for MinimumPendingRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRetentionProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRetentionProposalAdjuster for FlaggedPendingRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRetentionProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRetentionProposalAdjuster for TieredPendingRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRetentionProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending retention proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRetentionProposalEngine {
    pub steps: Vec<Rc<dyn PendingRetentionProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRetentionProposalEngine {
    pub fn run(&mut self, item: &PendingRetentionProposal) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingRetentionProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingRetentionProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_retention_proposal_engine(flags: &flags::Registry) -> PendingRetentionProposalEngine {
    PendingRetentionProposalEngine {
        steps: vec![
            Rc::new(RoundingPendingRetentionProposalAdjuster { step: 0.25 })
                as Rc<dyn PendingRetentionProposalAdjuster>,
            Rc::new(NamePendingRetentionProposalAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.22),
                    ("online".to_string(), 1.17),
                    ("east".to_string(), 1.35),
                ]),
            }),
            Rc::new(PendingRetentionProposalSequenceOffsetAdjuster { weight: 1.07 }),
            Rc::new(MinimumPendingRetentionProposalAdjuster { minimum: 49.1 }),
            Rc::new(FlaggedPendingRetentionProposalAdjuster {
                flags: flags.clone(),
                flag: "pending-retention-proposal-scheduled".to_string(),
                percent: -4.9,
            }),
            Rc::new(TieredPendingRetentionProposalAdjuster {
                bounds: vec![1395.0, 2808.0, 6796.0],
                rates: vec![1.08, 0.81, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 70617.4,
    }
}
