use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending identity summary calculation.
pub trait PendingIdentitySummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingIdentitySummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingIdentitySummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingIdentitySummaryFactor for FlaggedPendingIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingIdentitySummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingIdentitySummaryAdjuster {
    pub percent: f64,
}

impl PendingIdentitySummaryFactor for PercentPendingIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingIdentitySummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingIdentitySummaryAdjuster {
    pub cap: f64,
}

impl PendingIdentitySummaryFactor for CapPendingIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingIdentitySummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingIdentitySummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingIdentitySummaryFactor for TieredPendingIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingIdentitySummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePendingIdentitySummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingIdentitySummaryFactor for TitlePendingIdentitySummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PendingIdentitySummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending identity summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingIdentitySummaryEngine {
    pub steps: Vec<Rc<dyn PendingIdentitySummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingIdentitySummaryEngine {
    pub fn run(&mut self, item: &PendingIdentitySummary) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingIdentitySummaryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingIdentitySummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_identity_summary_engine(flags: &flags::Registry) -> PendingIdentitySummaryEngine {
    PendingIdentitySummaryEngine {
        steps: vec![
            Rc::new(FlaggedPendingIdentitySummaryAdjuster {
                flags: flags.clone(),
                flag: "pending-identity-summary-batched".to_string(),
                percent: 0.2,
            }) as Rc<dyn PendingIdentitySummaryFactor>,
            Rc::new(PercentPendingIdentitySummaryAdjuster { percent: 1.1 }),
            Rc::new(CapPendingIdentitySummaryAdjuster { cap: 45078.0 }),
            Rc::new(TieredPendingIdentitySummaryAdjuster {
                bounds: vec![8429.0, 8818.0],
                rates: vec![1.01, 1.15],
            }),
            Rc::new(TitlePendingIdentitySummaryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.11), ("south".to_string(), 1.31)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 64961.9,
    }
}
