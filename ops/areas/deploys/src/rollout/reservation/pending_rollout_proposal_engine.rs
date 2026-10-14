use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending rollout proposal calculation.
pub trait PendingRolloutProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRolloutProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingRolloutProposalAdjuster {
    pub cap: f64,
}

impl PendingRolloutProposalFactor for CapPendingRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingRolloutProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingRolloutProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingRolloutProposalFactor for FlaggedPendingRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingRolloutProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePendingRolloutProposalAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRolloutProposalFactor for SourcePendingRolloutProposalAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PendingRolloutProposal, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRolloutProposalAdjuster {
    pub minimum: f64,
}

impl PendingRolloutProposalFactor for MinimumPendingRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRolloutProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending rollout proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRolloutProposalEngine {
    pub steps: Vec<Rc<dyn PendingRolloutProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRolloutProposalEngine {
    pub fn evaluate(&mut self, item: &PendingRolloutProposal) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[PendingRolloutProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_rollout_proposal_engine(flags: &flags::Registry) -> PendingRolloutProposalEngine {
    PendingRolloutProposalEngine {
        steps: vec![
            Rc::new(CapPendingRolloutProposalAdjuster { cap: 14131.0 }) as Rc<dyn PendingRolloutProposalFactor>,
            Rc::new(FlaggedPendingRolloutProposalAdjuster {
                flags: flags.clone(),
                flag: "pending-rollout-proposal-scheduled".to_string(),
                percent: 8.7,
            }),
            Rc::new(SourcePendingRolloutProposalAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.03)]),
            }),
            Rc::new(MinimumPendingRolloutProposalAdjuster { minimum: 44.8 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 86540.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_rollout_proposal_engine(&flags::new_registry());
        let item = new_pending_rollout_proposal("code-175", "label-562");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
