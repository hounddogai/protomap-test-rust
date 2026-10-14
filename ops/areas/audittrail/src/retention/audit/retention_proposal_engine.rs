use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the retention proposal calculation.
pub trait RetentionProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRetentionProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RetentionProposalAdjuster for FlaggedRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRetentionProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RetentionProposalAdjuster for TieredRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionProposalAdjuster {
    pub minimum: f64,
}

impl RetentionProposalAdjuster for MinimumRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRetentionProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionProposalAdjuster for ChannelRetentionProposalAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionProposalAdjuster {
    pub percent: f64,
}

impl RetentionProposalAdjuster for PercentRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRetentionProposalAdjuster {
    pub cap: f64,
}

impl RetentionProposalAdjuster for CapRetentionProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RetentionProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the retention proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionProposalEngine {
    pub steps: Vec<Rc<dyn RetentionProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionProposalEngine {
    pub fn price(&mut self, item: &RetentionProposal) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionProposalAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_retention_proposal_engine(flags: &flags::Registry) -> RetentionProposalEngine {
    RetentionProposalEngine {
        steps: vec![
            Rc::new(FlaggedRetentionProposalAdjuster {
                flags: flags.clone(),
                flag: "retention-proposal-fallback".to_string(),
                percent: -4.7,
            }) as Rc<dyn RetentionProposalAdjuster>,
            Rc::new(TieredRetentionProposalAdjuster {
                bounds: vec![1299.0, 1447.0, 9573.0],
                rates: vec![0.82, 0.96, 0.9],
            }),
            Rc::new(MinimumRetentionProposalAdjuster { minimum: 27.5 }),
            Rc::new(ChannelRetentionProposalAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.94)]),
            }),
            Rc::new(PercentRetentionProposalAdjuster { percent: 14.2 }),
            Rc::new(CapRetentionProposalAdjuster { cap: 38328.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 26407.6,
    }
}
