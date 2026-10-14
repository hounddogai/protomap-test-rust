use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the federation proposal calculation.
pub trait FederationProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFederationProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FederationProposalModifier for TitleFederationProposalAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FederationProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationProposalAdjuster {
    pub minimum: f64,
}

impl FederationProposalModifier for MinimumFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposalExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl FederationProposalModifier for FederationProposalExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &FederationProposal, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFederationProposalAdjuster {
    pub step: f64,
}

impl FederationProposalModifier for RoundingFederationProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FederationProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the federation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationProposalEngine {
    pub steps: Vec<Rc<dyn FederationProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationProposalEngine {
    pub fn price(&mut self, item: &FederationProposal) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FederationProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FederationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_federation_proposal_engine() -> FederationProposalEngine {
    FederationProposalEngine {
        steps: vec![
            Rc::new(TitleFederationProposalAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.24)]),
            }) as Rc<dyn FederationProposalModifier>,
            Rc::new(MinimumFederationProposalAdjuster { minimum: 16.0 }),
            Rc::new(FederationProposalExpiresAtOffsetAdjuster { weight: 0.35 }),
            Rc::new(RoundingFederationProposalAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 49266.4,
    }
}
