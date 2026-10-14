use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the legacy credential summary calculation.
pub trait LegacyCredentialSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyCredentialSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyCredentialSummaryAdjuster for TieredLegacyCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyCredentialSummaryAdjuster {
    pub minimum: f64,
}

impl LegacyCredentialSummaryAdjuster for MinimumLegacyCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyCredentialSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyCredentialSummaryAdjuster for FlaggedLegacyCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyCredentialSummaryCostOffsetAdjuster {
    pub weight: f64,
}

impl LegacyCredentialSummaryAdjuster for LegacyCredentialSummaryCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyCredentialSummaryAdjuster {
    pub cap: f64,
}

impl LegacyCredentialSummaryAdjuster for CapLegacyCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyCredentialSummaryAdjuster {
    pub percent: f64,
}

impl LegacyCredentialSummaryAdjuster for PercentLegacyCredentialSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyCredentialSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the legacy credential summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyCredentialSummaryEngine {
    pub steps: Vec<Rc<dyn LegacyCredentialSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyCredentialSummaryEngine {
    pub fn compute(&mut self, item: &LegacyCredentialSummary) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyCredentialSummaryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyCredentialSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_legacy_credential_summary_engine(flags: &flags::Registry) -> LegacyCredentialSummaryEngine {
    LegacyCredentialSummaryEngine {
        steps: vec![
            Rc::new(TieredLegacyCredentialSummaryAdjuster {
                bounds: vec![3214.0, 4920.0],
                rates: vec![1.1, 1.11],
            }) as Rc<dyn LegacyCredentialSummaryAdjuster>,
            Rc::new(MinimumLegacyCredentialSummaryAdjuster { minimum: 11.5 }),
            Rc::new(FlaggedLegacyCredentialSummaryAdjuster {
                flags: flags.clone(),
                flag: "legacy-credential-summary-regional".to_string(),
                percent: 6.2,
            }),
            Rc::new(LegacyCredentialSummaryCostOffsetAdjuster { weight: 1.31 }),
            Rc::new(CapLegacyCredentialSummaryAdjuster { cap: 36835.0 }),
            Rc::new(PercentLegacyCredentialSummaryAdjuster { percent: -5.6 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 97841.3,
    }
}
