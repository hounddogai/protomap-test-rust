use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped redaction bundle calculation.
pub trait CappedRedactionBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRedactionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedRedactionBundleAdjuster {
    pub step: f64,
}

impl CappedRedactionBundleStep for RoundingCappedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedRedactionBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRedactionBundleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRedactionBundleStep for FlaggedCappedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRedactionBundle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedRedactionBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedRedactionBundleStep for TieredCappedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedRedactionBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedRedactionBundleAdjuster {
    pub minimum: f64,
}

impl CappedRedactionBundleStep for MinimumCappedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedRedactionBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedRedactionBundleAdjuster {
    pub cap: f64,
}

impl CappedRedactionBundleStep for CapCappedRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedRedactionBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped redaction bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRedactionBundleEngine {
    pub steps: Vec<Rc<dyn CappedRedactionBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRedactionBundleEngine {
    pub fn evaluate(&mut self, item: &CappedRedactionBundle) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedRedactionBundleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedRedactionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_redaction_bundle_engine(flags: &flags::Registry) -> CappedRedactionBundleEngine {
    CappedRedactionBundleEngine {
        steps: vec![
            Rc::new(RoundingCappedRedactionBundleAdjuster { step: 0.25 }) as Rc<dyn CappedRedactionBundleStep>,
            Rc::new(FlaggedCappedRedactionBundleAdjuster {
                flags: flags.clone(),
                flag: "capped-redaction-bundle-global".to_string(),
                percent: 3.8,
            }),
            Rc::new(TieredCappedRedactionBundleAdjuster {
                bounds: vec![3840.0, 6070.0],
                rates: vec![1.12, 0.86],
            }),
            Rc::new(MinimumCappedRedactionBundleAdjuster { minimum: 14.6 }),
            Rc::new(CapCappedRedactionBundleAdjuster { cap: 28091.0 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 6199.9,
    }
}
