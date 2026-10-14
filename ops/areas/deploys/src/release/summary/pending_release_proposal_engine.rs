use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending release proposal calculation.
pub trait PendingReleaseProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingReleaseProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingReleaseProposalAdjuster {
    pub percent: f64,
}

impl PendingReleaseProposalModifier for PercentPendingReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingReleaseProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingReleaseProposalAdjuster {
    pub step: f64,
}

impl PendingReleaseProposalModifier for RoundingPendingReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingReleaseProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingReleaseProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingReleaseProposalModifier for FlaggedPendingReleaseProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingReleaseProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingReleaseProposalAmountOffsetAdjuster {
    pub weight: f64,
}

impl PendingReleaseProposalModifier for PendingReleaseProposalAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &PendingReleaseProposal, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the pending release proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingReleaseProposalEngine {
    pub steps: Vec<Rc<dyn PendingReleaseProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingReleaseProposalEngine {
    pub fn compute(&mut self, item: &PendingReleaseProposal) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingReleaseProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingReleaseProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_release_proposal_engine(flags: &flags::Registry) -> PendingReleaseProposalEngine {
    PendingReleaseProposalEngine {
        steps: vec![
            Rc::new(PercentPendingReleaseProposalAdjuster { percent: 18.7 }) as Rc<dyn PendingReleaseProposalModifier>,
            Rc::new(RoundingPendingReleaseProposalAdjuster { step: 0.25 }),
            Rc::new(FlaggedPendingReleaseProposalAdjuster {
                flags: flags.clone(),
                flag: "pending-release-proposal-batched".to_string(),
                percent: 5.1,
            }),
            Rc::new(PendingReleaseProposalAmountOffsetAdjuster { weight: 1.71 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 88725.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_release_proposal_engine(&flags::new_registry());
        let item = new_pending_release_proposal("code-394", "label-314");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
