use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary escalation proposal calculation.
pub trait PrimaryEscalationProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEscalationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryEscalationProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryEscalationProposalModifier for FlaggedPrimaryEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryEscalationProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryEscalationProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryEscalationProposalModifier for TieredPrimaryEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryEscalationProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePrimaryEscalationProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEscalationProposalModifier for NamePrimaryEscalationProposalAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PrimaryEscalationProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryEscalationProposalAdjuster {
    pub cap: f64,
}

impl PrimaryEscalationProposalModifier for CapPrimaryEscalationProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryEscalationProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary escalation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEscalationProposalEngine {
    pub steps: Vec<Rc<dyn PrimaryEscalationProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEscalationProposalEngine {
    pub fn price(&mut self, item: &PrimaryEscalationProposal) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryEscalationProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEscalationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_escalation_proposal_engine(flags: &flags::Registry) -> PrimaryEscalationProposalEngine {
    PrimaryEscalationProposalEngine {
        steps: vec![
            Rc::new(FlaggedPrimaryEscalationProposalAdjuster {
                flags: flags.clone(),
                flag: "primary-escalation-proposal-regional".to_string(),
                percent: 4.8,
            }) as Rc<dyn PrimaryEscalationProposalModifier>,
            Rc::new(TieredPrimaryEscalationProposalAdjuster {
                bounds: vec![1184.0, 2047.0, 8736.0, 9157.0],
                rates: vec![1.01, 0.96, 1.12, 1.09],
            }),
            Rc::new(NamePrimaryEscalationProposalAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.96), ("central".to_string(), 1.03)]),
            }),
            Rc::new(CapPrimaryEscalationProposalAdjuster { cap: 31319.0 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 46456.0,
    }
}
