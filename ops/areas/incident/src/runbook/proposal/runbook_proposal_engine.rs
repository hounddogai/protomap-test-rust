use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook proposal calculation.
pub trait RunbookProposalModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookProposalAdjuster {
    pub cap: f64,
}

impl RunbookProposalModifier for CapRunbookProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookProposalAdjuster {
    pub percent: f64,
}

impl RunbookProposalModifier for PercentRunbookProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookProposalAdjuster {
    pub step: f64,
}

impl RunbookProposalModifier for RoundingRunbookProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookProposalModifier for FlaggedRunbookProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookProposalPriorityOffsetAdjuster {
    pub weight: f64,
}

impl RunbookProposalModifier for RunbookProposalPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookProposal, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the runbook proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookProposalEngine {
    pub steps: Vec<Rc<dyn RunbookProposalModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookProposalEngine {
    pub fn run(&mut self, item: &RunbookProposal) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RunbookProposalModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RunbookProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_proposal_engine(flags: &flags::Registry) -> RunbookProposalEngine {
    RunbookProposalEngine {
        steps: vec![
            Rc::new(CapRunbookProposalAdjuster { cap: 23667.0 }) as Rc<dyn RunbookProposalModifier>,
            Rc::new(PercentRunbookProposalAdjuster { percent: 13.1 }),
            Rc::new(RoundingRunbookProposalAdjuster { step: 0.05 }),
            Rc::new(FlaggedRunbookProposalAdjuster {
                flags: flags.clone(),
                flag: "runbook-proposal-scheduled".to_string(),
                percent: -9.3,
            }),
            Rc::new(RunbookProposalPriorityOffsetAdjuster { weight: 1.31 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 82661.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_proposal_engine(&flags::new_registry());
        let item = new_runbook_proposal("code-907", "label-681");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
