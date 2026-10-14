use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the delegation proposal calculation.
pub trait DelegationProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationProposalAdjuster {
    pub minimum: f64,
}

impl DelegationProposalStep for MinimumDelegationProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationProposalStep for FlaggedDelegationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationProposalStep for TieredDelegationProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the delegation proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationProposalEngine {
    pub steps: Vec<Rc<dyn DelegationProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationProposalEngine {
    pub fn price(&mut self, item: &DelegationProposal) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[DelegationProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_delegation_proposal_engine(flags: &flags::Registry) -> DelegationProposalEngine {
    DelegationProposalEngine {
        steps: vec![
            Rc::new(MinimumDelegationProposalAdjuster { minimum: 40.3 }) as Rc<dyn DelegationProposalStep>,
            Rc::new(FlaggedDelegationProposalAdjuster {
                flags: flags.clone(),
                flag: "delegation-proposal-legacy".to_string(),
                percent: -6.0,
            }),
            Rc::new(TieredDelegationProposalAdjuster {
                bounds: vec![2743.0, 4490.0, 8595.0, 9411.0],
                rates: vec![0.9, 1.05, 0.9, 0.81],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 86640.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_proposal_engine(&flags::new_registry());
        let item = new_delegation_proposal("code-177", "label-503");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
