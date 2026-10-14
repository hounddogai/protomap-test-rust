use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive federation proposal calculation.
pub trait AdaptiveFederationProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveFederationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeAdaptiveFederationProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveFederationProposalModifier for ScopeAdaptiveFederationProposalAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &AdaptiveFederationProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveFederationProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveFederationProposalModifier for FlaggedAdaptiveFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveFederationProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveFederationProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveFederationProposalModifier for TieredAdaptiveFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveFederationProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive federation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveFederationProposalEngine {
    pub steps: Vec<Rc<dyn AdaptiveFederationProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveFederationProposalEngine {
    pub fn evaluate(&mut self, item: &AdaptiveFederationProposal) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveFederationProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveFederationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_adaptive_federation_proposal_engine(flags: &flags::Registry) -> AdaptiveFederationProposalEngine {
    AdaptiveFederationProposalEngine {
        steps: vec![
            Rc::new(ScopeAdaptiveFederationProposalAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.71), ("central".to_string(), 0.8)]),
            }) as Rc<dyn AdaptiveFederationProposalModifier>,
            Rc::new(FlaggedAdaptiveFederationProposalAdjuster {
                flags: flags.clone(),
                flag: "adaptive-federation-proposal-tiered".to_string(),
                percent: -0.1,
            }),
            Rc::new(TieredAdaptiveFederationProposalAdjuster {
                bounds: vec![37.0, 4197.0, 7146.0, 7423.0],
                rates: vec![1.16, 0.87, 0.97, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 91912.8,
    }
}
