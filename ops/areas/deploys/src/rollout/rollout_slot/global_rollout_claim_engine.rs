use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global rollout claim calculation.
pub trait GlobalRolloutClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryGlobalRolloutClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalRolloutClaimAdjuster for CountryGlobalRolloutClaimAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalRolloutClaimAdjuster {
    pub step: f64,
}

impl GlobalRolloutClaimAdjuster for RoundingGlobalRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalRolloutClaimAdjuster {
    pub cap: f64,
}

impl GlobalRolloutClaimAdjuster for CapGlobalRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalRolloutClaimAdjuster {
    pub percent: f64,
}

impl GlobalRolloutClaimAdjuster for PercentGlobalRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalRolloutClaimAdjuster {
    pub minimum: f64,
}

impl GlobalRolloutClaimAdjuster for MinimumGlobalRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRolloutClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRolloutClaimAdjuster for FlaggedGlobalRolloutClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRolloutClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global rollout claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRolloutClaimEngine {
    pub steps: Vec<Rc<dyn GlobalRolloutClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRolloutClaimEngine {
    pub fn price(&mut self, item: &GlobalRolloutClaim) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalRolloutClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRolloutClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_rollout_claim_engine(flags: &flags::Registry) -> GlobalRolloutClaimEngine {
    GlobalRolloutClaimEngine {
        steps: vec![
            Rc::new(CountryGlobalRolloutClaimAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.74), ("central".to_string(), 1.03)]),
            }) as Rc<dyn GlobalRolloutClaimAdjuster>,
            Rc::new(RoundingGlobalRolloutClaimAdjuster { step: 0.1 }),
            Rc::new(CapGlobalRolloutClaimAdjuster { cap: 2913.0 }),
            Rc::new(PercentGlobalRolloutClaimAdjuster { percent: -21.6 }),
            Rc::new(MinimumGlobalRolloutClaimAdjuster { minimum: 19.5 }),
            Rc::new(FlaggedGlobalRolloutClaimAdjuster {
                flags: flags.clone(),
                flag: "global-rollout-claim-rolling".to_string(),
                percent: -9.0,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 43385.9,
    }
}
