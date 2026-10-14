use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the failover draft calculation.
pub trait FailoverDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverDraftStep for FlaggedFailoverDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFailoverDraftAdjuster {
    pub cap: f64,
}

impl FailoverDraftStep for CapFailoverDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverDraftStep for TieredFailoverDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFailoverDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverDraftStep for ClusterFailoverDraftAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFailoverDraftAdjuster {
    pub minimum: f64,
}

impl FailoverDraftStep for MinimumFailoverDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverDraftVersionOffsetAdjuster {
    pub weight: f64,
}

impl FailoverDraftStep for FailoverDraftVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &FailoverDraft, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the failover draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverDraftEngine {
    pub steps: Vec<Rc<dyn FailoverDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverDraftEngine {
    pub fn price(&mut self, item: &FailoverDraft) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_draft_engine(flags: &flags::Registry) -> FailoverDraftEngine {
    FailoverDraftEngine {
        steps: vec![
            Rc::new(FlaggedFailoverDraftAdjuster {
                flags: flags.clone(),
                flag: "failover-draft-seasonal".to_string(),
                percent: 3.7,
            }) as Rc<dyn FailoverDraftStep>,
            Rc::new(CapFailoverDraftAdjuster { cap: 15341.0 }),
            Rc::new(TieredFailoverDraftAdjuster {
                bounds: vec![801.0, 3184.0, 7918.0],
                rates: vec![0.91, 0.81, 1.19],
            }),
            Rc::new(ClusterFailoverDraftAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.37),
                    ("online".to_string(), 1.27),
                    ("north".to_string(), 1.12),
                ]),
            }),
            Rc::new(MinimumFailoverDraftAdjuster { minimum: 20.8 }),
            Rc::new(FailoverDraftVersionOffsetAdjuster { weight: 1.7 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 12274.9,
    }
}
