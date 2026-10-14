use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage proposal calculation.
pub trait UsageProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageProposalAdjuster {
    pub step: f64,
}

impl UsageProposalFactor for RoundingUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageProposalAdjuster {
    pub percent: f64,
}

impl UsageProposalFactor for PercentUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl UsageProposalFactor for TieredUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &UsageProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageProposalAdjuster {
    pub minimum: f64,
}

impl UsageProposalFactor for MinimumUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapUsageProposalAdjuster {
    pub cap: f64,
}

impl UsageProposalFactor for CapUsageProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &UsageProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeUsageProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageProposalFactor for ScopeUsageProposalAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &UsageProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the usage proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageProposalEngine {
    pub steps: Vec<Rc<dyn UsageProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageProposalEngine {
    pub fn run(&mut self, item: &UsageProposal) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn UsageProposalFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[UsageProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_usage_proposal_engine() -> UsageProposalEngine {
    UsageProposalEngine {
        steps: vec![
            Rc::new(RoundingUsageProposalAdjuster { step: 0.1 }) as Rc<dyn UsageProposalFactor>,
            Rc::new(PercentUsageProposalAdjuster { percent: -8.6 }),
            Rc::new(TieredUsageProposalAdjuster {
                bounds: vec![637.0, 926.0, 8995.0, 9418.0],
                rates: vec![1.18, 1.12, 1.09, 1.07],
            }),
            Rc::new(MinimumUsageProposalAdjuster { minimum: 41.5 }),
            Rc::new(CapUsageProposalAdjuster { cap: 42908.0 }),
            Rc::new(ScopeUsageProposalAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.29), ("online".to_string(), 1.0)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 45803.2,
    }
}
