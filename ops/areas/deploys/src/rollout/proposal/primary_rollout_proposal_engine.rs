use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary rollout proposal calculation.
pub trait PrimaryRolloutProposalFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryRolloutProposalAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryRolloutProposalFactor for TieredPrimaryRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRolloutProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRolloutProposalFactor for FlaggedPrimaryRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposalUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryRolloutProposalFactor for PrimaryRolloutProposalUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryRolloutProposalAdjuster {
    pub step: f64,
}

impl PrimaryRolloutProposalFactor for RoundingPrimaryRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryRolloutProposalAdjuster {
    pub minimum: f64,
}

impl PrimaryRolloutProposalFactor for MinimumPrimaryRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRolloutProposalAdjuster {
    pub cap: f64,
}

impl PrimaryRolloutProposalFactor for CapPrimaryRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRolloutProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the primary rollout proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutProposalEngine {
    pub steps: Vec<Rc<dyn PrimaryRolloutProposalFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRolloutProposalEngine {
    pub fn run(&mut self, item: &PrimaryRolloutProposal) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryRolloutProposalFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRolloutProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_rollout_proposal_engine(flags: &flags::Registry) -> PrimaryRolloutProposalEngine {
    PrimaryRolloutProposalEngine {
        steps: vec![
            Rc::new(TieredPrimaryRolloutProposalAdjuster {
                bounds: vec![319.0, 669.0],
                rates: vec![1.06, 0.99],
            }) as Rc<dyn PrimaryRolloutProposalFactor>,
            Rc::new(FlaggedPrimaryRolloutProposalAdjuster {
                flags: flags.clone(),
                flag: "primary-rollout-proposal-strict".to_string(),
                percent: -9.0,
            }),
            Rc::new(PrimaryRolloutProposalUpdatedAtOffsetAdjuster { weight: 1.52 }),
            Rc::new(RoundingPrimaryRolloutProposalAdjuster { step: 0.1 }),
            Rc::new(MinimumPrimaryRolloutProposalAdjuster { minimum: 18.8 }),
            Rc::new(CapPrimaryRolloutProposalAdjuster { cap: 9707.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 29840.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_rollout_proposal_engine(&flags::new_registry());
        let item = new_primary_rollout_proposal("code-194", "label-915");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
