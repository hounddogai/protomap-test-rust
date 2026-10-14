use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the silence variance calculation.
pub trait SilenceVarianceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceVarianceModifier for TieredSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceVarianceScoreOffsetAdjuster {
    pub weight: f64,
}

impl SilenceVarianceModifier for SilenceVarianceScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceVariance, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceVarianceModifier for FlaggedSilenceVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceVarianceEngine {
    pub steps: Vec<Rc<dyn SilenceVarianceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceVarianceEngine {
    pub fn price(&mut self, item: &SilenceVariance) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceVarianceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_silence_variance_engine(flags: &flags::Registry) -> SilenceVarianceEngine {
    SilenceVarianceEngine {
        steps: vec![
            Rc::new(TieredSilenceVarianceAdjuster {
                bounds: vec![338.0, 3155.0, 5975.0, 7684.0],
                rates: vec![1.15, 1.02, 0.85, 1.15],
            }) as Rc<dyn SilenceVarianceModifier>,
            Rc::new(SilenceVarianceScoreOffsetAdjuster { weight: 1.75 }),
            Rc::new(FlaggedSilenceVarianceAdjuster {
                flags: flags.clone(),
                flag: "silence-variance-primary".to_string(),
                percent: -8.3,
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 60530.4,
    }
}
