use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient ttl policy proposal calculation.
pub trait LenientTtlPolicyProposalAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientTtlPolicyProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientTtlPolicyProposalAdjuster {
    pub step: f64,
}

impl LenientTtlPolicyProposalAdjuster for RoundingLenientTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientTtlPolicyProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientTtlPolicyProposalAdjuster {
    pub cap: f64,
}

impl LenientTtlPolicyProposalAdjuster for CapLenientTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientTtlPolicyProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientTtlPolicyProposalAdjuster {
    pub minimum: f64,
}

impl LenientTtlPolicyProposalAdjuster for MinimumLenientTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientTtlPolicyProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientTtlPolicyProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientTtlPolicyProposalAdjuster for FlaggedLenientTtlPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientTtlPolicyProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient ttl policy proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientTtlPolicyProposalEngine {
    pub steps: Vec<Rc<dyn LenientTtlPolicyProposalAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientTtlPolicyProposalEngine {
    pub fn compute(&mut self, item: &LenientTtlPolicyProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[LenientTtlPolicyProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_ttl_policy_proposal_engine(flags: &flags::Registry) -> LenientTtlPolicyProposalEngine {
    LenientTtlPolicyProposalEngine {
        steps: vec![
            Rc::new(RoundingLenientTtlPolicyProposalAdjuster { step: 0.25 })
                as Rc<dyn LenientTtlPolicyProposalAdjuster>,
            Rc::new(CapLenientTtlPolicyProposalAdjuster { cap: 21861.0 }),
            Rc::new(MinimumLenientTtlPolicyProposalAdjuster { minimum: 0.0 }),
            Rc::new(FlaggedLenientTtlPolicyProposalAdjuster {
                flags: flags.clone(),
                flag: "lenient-ttl-policy-proposal-adaptive".to_string(),
                percent: 5.7,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 33916.7,
    }
}
