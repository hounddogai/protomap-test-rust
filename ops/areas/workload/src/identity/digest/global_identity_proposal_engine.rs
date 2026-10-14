use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global identity proposal calculation.
pub trait GlobalIdentityProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalIdentityProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalIdentityProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalIdentityProposalFactor for FlaggedGlobalIdentityProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalIdentityProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalIdentityProposalAdjuster {
    pub step: f64,
}

impl GlobalIdentityProposalFactor for RoundingGlobalIdentityProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalIdentityProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalIdentityProposalAdjuster {
    pub percent: f64,
}

impl GlobalIdentityProposalFactor for PercentGlobalIdentityProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalIdentityProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global identity proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalIdentityProposalEngine {
    pub steps: Vec<Rc<dyn GlobalIdentityProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalIdentityProposalEngine {
    pub fn price(&mut self, item: &GlobalIdentityProposal) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[GlobalIdentityProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_identity_proposal_engine(flags: &flags::Registry) -> GlobalIdentityProposalEngine {
    GlobalIdentityProposalEngine {
        steps: vec![
            Rc::new(FlaggedGlobalIdentityProposalAdjuster {
                flags: flags.clone(),
                flag: "global-identity-proposal-batched".to_string(),
                percent: 9.9,
            }) as Rc<dyn GlobalIdentityProposalFactor>,
            Rc::new(RoundingGlobalIdentityProposalAdjuster { step: 5.0 }),
            Rc::new(PercentGlobalIdentityProposalAdjuster { percent: 2.2 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 92006.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_identity_proposal_engine(&flags::new_registry());
        let item = new_global_identity_proposal("code-682", "title-144");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
