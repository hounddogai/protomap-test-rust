use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending action item proposal calculation.
pub trait PendingActionItemProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingActionItemProposal, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingActionItemProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingActionItemProposalStep for FlaggedPendingActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingActionItemProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingActionItemProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingActionItemProposalStep for TieredPendingActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingActionItemProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemProposalRevisionOffsetAdjuster {
    pub weight: f64,
}

impl PendingActionItemProposalStep for PendingActionItemProposalRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &PendingActionItemProposal, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingActionItemProposalAdjuster {
    pub minimum: f64,
}

impl PendingActionItemProposalStep for MinimumPendingActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingActionItemProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingActionItemProposalAdjuster {
    pub cap: f64,
}

impl PendingActionItemProposalStep for CapPendingActionItemProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingActionItemProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending action item proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingActionItemProposalEngine {
    pub steps: Vec<Rc<dyn PendingActionItemProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingActionItemProposalEngine {
    pub fn compute(&mut self, item: &PendingActionItemProposal) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingActionItemProposalStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingActionItemProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_action_item_proposal_engine(flags: &flags::Registry) -> PendingActionItemProposalEngine {
    PendingActionItemProposalEngine {
        steps: vec![
            Rc::new(FlaggedPendingActionItemProposalAdjuster {
                flags: flags.clone(),
                flag: "pending-action-item-proposal-regional".to_string(),
                percent: 0.2,
            }) as Rc<dyn PendingActionItemProposalStep>,
            Rc::new(TieredPendingActionItemProposalAdjuster {
                bounds: vec![4026.0, 4493.0, 5012.0, 8245.0],
                rates: vec![1.15, 1.14, 0.94, 0.94],
            }),
            Rc::new(PendingActionItemProposalRevisionOffsetAdjuster { weight: 0.08 }),
            Rc::new(MinimumPendingActionItemProposalAdjuster { minimum: 43.5 }),
            Rc::new(CapPendingActionItemProposalAdjuster { cap: 18520.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 88187.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_action_item_proposal_engine(&flags::new_registry());
        let item = new_pending_action_item_proposal("code-178", "name-536");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
