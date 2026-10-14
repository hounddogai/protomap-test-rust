use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary egress proposal calculation.
pub trait PrimaryEgressProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryEgressProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEgressProposalMarginOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryEgressProposalStep for PrimaryEgressProposalMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryEgressProposal, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePrimaryEgressProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryEgressProposalStep for NamePrimaryEgressProposalAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PrimaryEgressProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryEgressProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryEgressProposalStep for FlaggedPrimaryEgressProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryEgressProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryEgressProposalAdjuster {
    pub cap: f64,
}

impl PrimaryEgressProposalStep for CapPrimaryEgressProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryEgressProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary egress proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryEgressProposalEngine {
    pub steps: Vec<Rc<dyn PrimaryEgressProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryEgressProposalEngine {
    pub fn run(&mut self, item: &PrimaryEgressProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryEgressProposalStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryEgressProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_egress_proposal_engine(flags: &flags::Registry) -> PrimaryEgressProposalEngine {
    PrimaryEgressProposalEngine {
        steps: vec![
            Rc::new(PrimaryEgressProposalMarginOffsetAdjuster { weight: 0.21 }) as Rc<dyn PrimaryEgressProposalStep>,
            Rc::new(NamePrimaryEgressProposalAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.05)]),
            }),
            Rc::new(FlaggedPrimaryEgressProposalAdjuster {
                flags: flags.clone(),
                flag: "primary-egress-proposal-adaptive".to_string(),
                percent: 6.4,
            }),
            Rc::new(CapPrimaryEgressProposalAdjuster { cap: 48744.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 97041.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_egress_proposal_engine(&flags::new_registry());
        let item = new_primary_egress_proposal("code-182", "name-983");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
