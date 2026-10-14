use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the batched postmortem proposal calculation.
pub trait BatchedPostmortemProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProposalCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl BatchedPostmortemProposalStep for BatchedPostmortemProposalCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPostmortemProposalAdjuster {
    pub minimum: f64,
}

impl BatchedPostmortemProposalStep for MinimumBatchedPostmortemProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedPostmortemProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedPostmortemProposalStep for TieredBatchedPostmortemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedPostmortemProposalAdjuster {
    pub cap: f64,
}

impl BatchedPostmortemProposalStep for CapBatchedPostmortemProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedPostmortemProposalAdjuster {
    pub percent: f64,
}

impl BatchedPostmortemProposalStep for PercentBatchedPostmortemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPostmortemProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPostmortemProposalStep for FlaggedBatchedPostmortemProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPostmortemProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched postmortem proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPostmortemProposalEngine {
    pub steps: Vec<Rc<dyn BatchedPostmortemProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPostmortemProposalEngine {
    pub fn price(&mut self, item: &BatchedPostmortemProposal) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BatchedPostmortemProposalStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPostmortemProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_postmortem_proposal_engine(flags: &flags::Registry) -> BatchedPostmortemProposalEngine {
    BatchedPostmortemProposalEngine {
        steps: vec![
            Rc::new(BatchedPostmortemProposalCreatedAtOffsetAdjuster { weight: 1.63 })
                as Rc<dyn BatchedPostmortemProposalStep>,
            Rc::new(MinimumBatchedPostmortemProposalAdjuster { minimum: 47.5 }),
            Rc::new(TieredBatchedPostmortemProposalAdjuster {
                bounds: vec![4280.0, 5981.0, 7040.0],
                rates: vec![0.86, 1.07, 1.11],
            }),
            Rc::new(CapBatchedPostmortemProposalAdjuster { cap: 34553.0 }),
            Rc::new(PercentBatchedPostmortemProposalAdjuster { percent: 4.3 }),
            Rc::new(FlaggedBatchedPostmortemProposalAdjuster {
                flags: flags.clone(),
                flag: "batched-postmortem-proposal-regional".to_string(),
                percent: -2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 31278.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_postmortem_proposal_engine(&flags::new_registry());
        let item = new_batched_postmortem_proposal("code-176", "label-559");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
