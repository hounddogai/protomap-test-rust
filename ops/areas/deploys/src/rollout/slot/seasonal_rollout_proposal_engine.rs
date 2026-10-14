use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal rollout proposal calculation.
pub trait SeasonalRolloutProposalStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalRolloutProposalAdjuster {
    pub step: f64,
}

impl SeasonalRolloutProposalStep for RoundingSeasonalRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalRolloutProposalAdjuster {
    pub minimum: f64,
}

impl SeasonalRolloutProposalStep for MinimumSeasonalRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalRolloutProposalAdjuster {
    pub cap: f64,
}

impl SeasonalRolloutProposalStep for CapSeasonalRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalRolloutProposalAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalRolloutProposalStep for FlaggedSeasonalRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalRolloutProposalAdjuster {
    pub percent: f64,
}

impl SeasonalRolloutProposalStep for PercentSeasonalRolloutProposalAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRolloutProposalRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalRolloutProposalStep for SeasonalRolloutProposalRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalRolloutProposal, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

/// Runs the seasonal rollout proposal steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalRolloutProposalEngine {
    pub steps: Vec<Rc<dyn SeasonalRolloutProposalStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalRolloutProposalEngine {
    pub fn price(&mut self, item: &SeasonalRolloutProposal) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalRolloutProposal]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_rollout_proposal_engine(flags: &flags::Registry) -> SeasonalRolloutProposalEngine {
    SeasonalRolloutProposalEngine {
        steps: vec![
            Rc::new(RoundingSeasonalRolloutProposalAdjuster { step: 5.0 }) as Rc<dyn SeasonalRolloutProposalStep>,
            Rc::new(MinimumSeasonalRolloutProposalAdjuster { minimum: 9.7 }),
            Rc::new(CapSeasonalRolloutProposalAdjuster { cap: 48079.0 }),
            Rc::new(FlaggedSeasonalRolloutProposalAdjuster {
                flags: flags.clone(),
                flag: "seasonal-rollout-proposal-adaptive".to_string(),
                percent: -9.6,
            }),
            Rc::new(PercentSeasonalRolloutProposalAdjuster { percent: -8.9 }),
            Rc::new(SeasonalRolloutProposalRotationDaysOffsetAdjuster { weight: 1.01 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 93200.9,
    }
}
