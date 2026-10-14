use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending allowlist claim calculation.
pub trait PendingAllowlistClaimStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingAllowlistClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingAllowlistClaimAdjuster {
    pub percent: f64,
}

impl PendingAllowlistClaimStep for PercentPendingAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingAllowlistClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingAllowlistClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingAllowlistClaimStep for FlaggedPendingAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingAllowlistClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingAllowlistClaimAdjuster {
    pub cap: f64,
}

impl PendingAllowlistClaimStep for CapPendingAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingAllowlistClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistClaimRetriesOffsetAdjuster {
    pub weight: f64,
}

impl PendingAllowlistClaimStep for PendingAllowlistClaimRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &PendingAllowlistClaim, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingAllowlistClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingAllowlistClaimStep for TieredPendingAllowlistClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingAllowlistClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending allowlist claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingAllowlistClaimEngine {
    pub steps: Vec<Rc<dyn PendingAllowlistClaimStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingAllowlistClaimEngine {
    pub fn evaluate(&mut self, item: &PendingAllowlistClaim) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingAllowlistClaimStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingAllowlistClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_allowlist_claim_engine(flags: &flags::Registry) -> PendingAllowlistClaimEngine {
    PendingAllowlistClaimEngine {
        steps: vec![
            Rc::new(PercentPendingAllowlistClaimAdjuster { percent: -9.0 }) as Rc<dyn PendingAllowlistClaimStep>,
            Rc::new(FlaggedPendingAllowlistClaimAdjuster {
                flags: flags.clone(),
                flag: "pending-allowlist-claim-seasonal".to_string(),
                percent: -1.8,
            }),
            Rc::new(CapPendingAllowlistClaimAdjuster { cap: 3311.0 }),
            Rc::new(PendingAllowlistClaimRetriesOffsetAdjuster { weight: 0.14 }),
            Rc::new(TieredPendingAllowlistClaimAdjuster {
                bounds: vec![5242.0, 9468.0],
                rates: vec![0.94, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 12291.6,
    }
}
