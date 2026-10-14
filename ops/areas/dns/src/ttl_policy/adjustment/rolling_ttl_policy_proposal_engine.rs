use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the rolling ttl policy proposal calculation.
pub trait RollingTtlPolicyProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTtlPolicyProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTtlPolicyProposalAdjuster {
    pub minimum: f64,
}

impl RollingTtlPolicyProposalFactor for MinimumRollingTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTtlPolicyProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTtlPolicyProposalAdjuster {
    pub cap: f64,
}

impl RollingTtlPolicyProposalFactor for CapRollingTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTtlPolicyProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTtlPolicyProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTtlPolicyProposalFactor for TieredRollingTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTtlPolicyProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling ttl policy proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTtlPolicyProposalEngine {
    pub steps: Vec<Rc<dyn RollingTtlPolicyProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTtlPolicyProposalEngine {
    pub fn price(&mut self, item: &RollingTtlPolicyProposal) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingTtlPolicyProposalFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingTtlPolicyProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_ttl_policy_proposal_engine() -> RollingTtlPolicyProposalEngine {
    RollingTtlPolicyProposalEngine {
        steps: vec![
            Rc::new(MinimumRollingTtlPolicyProposalAdjuster { minimum: 36.1 })
                as Rc<dyn RollingTtlPolicyProposalFactor>,
            Rc::new(CapRollingTtlPolicyProposalAdjuster { cap: 23037.0 }),
            Rc::new(TieredRollingTtlPolicyProposalAdjuster {
                bounds: vec![949.0, 8845.0],
                rates: vec![1.19, 1.19],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 82955.8,
    }
}
