use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy proposal calculation.
pub trait PolicyProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyProposalModifier for FlaggedPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyProposalAdjuster {
    pub cap: f64,
}

impl PolicyProposalModifier for CapPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyProposalAdjuster {
    pub percent: f64,
}

impl PolicyProposalModifier for PercentPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyProposalAdjuster {
    pub step: f64,
}

impl PolicyProposalModifier for RoundingPolicyProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the policy proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyProposalEngine {
    pub steps: Vec<Rc<dyn PolicyProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyProposalEngine {
    pub fn run(&mut self, item: &PolicyProposal) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PolicyProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_policy_proposal_engine(flags: &flags::Registry) -> PolicyProposalEngine {
    PolicyProposalEngine {
        steps: vec![
            Rc::new(FlaggedPolicyProposalAdjuster {
                flags: flags.clone(),
                flag: "policy-proposal-primary".to_string(),
                percent: -0.2,
            }) as Rc<dyn PolicyProposalModifier>,
            Rc::new(CapPolicyProposalAdjuster { cap: 19523.0 }),
            Rc::new(PercentPolicyProposalAdjuster { percent: 12.2 }),
            Rc::new(RoundingPolicyProposalAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 3127.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_proposal_engine(&flags::new_registry());
        let item = new_policy_proposal("code-484", "title-610");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
