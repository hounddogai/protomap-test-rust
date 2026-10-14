use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy measurement baseline calculation.
pub trait LegacyMeasurementBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyMeasurementBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLegacyMeasurementBaselineAdjuster {
    pub percent: f64,
}

impl LegacyMeasurementBaselineModifier for PercentLegacyMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LegacyMeasurementBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyMeasurementBaselineAdjuster {
    pub minimum: f64,
}

impl LegacyMeasurementBaselineModifier for MinimumLegacyMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyMeasurementBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyMeasurementBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyMeasurementBaselineModifier for FlaggedLegacyMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyMeasurementBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyMeasurementBaselineAdjuster {
    pub step: f64,
}

impl LegacyMeasurementBaselineModifier for RoundingLegacyMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyMeasurementBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyMeasurementBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyMeasurementBaselineModifier for TieredLegacyMeasurementBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyMeasurementBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the legacy measurement baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyMeasurementBaselineEngine {
    pub steps: Vec<Rc<dyn LegacyMeasurementBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyMeasurementBaselineEngine {
    pub fn price(&mut self, item: &LegacyMeasurementBaseline) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyMeasurementBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyMeasurementBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_measurement_baseline_engine(flags: &flags::Registry) -> LegacyMeasurementBaselineEngine {
    LegacyMeasurementBaselineEngine {
        steps: vec![
            Rc::new(PercentLegacyMeasurementBaselineAdjuster { percent: -21.7 })
                as Rc<dyn LegacyMeasurementBaselineModifier>,
            Rc::new(MinimumLegacyMeasurementBaselineAdjuster { minimum: 47.4 }),
            Rc::new(FlaggedLegacyMeasurementBaselineAdjuster {
                flags: flags.clone(),
                flag: "legacy-measurement-baseline-manual".to_string(),
                percent: 8.8,
            }),
            Rc::new(RoundingLegacyMeasurementBaselineAdjuster { step: 0.25 }),
            Rc::new(TieredLegacyMeasurementBaselineAdjuster {
                bounds: vec![1014.0, 6140.0],
                rates: vec![0.97, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 54599.4,
    }
}
