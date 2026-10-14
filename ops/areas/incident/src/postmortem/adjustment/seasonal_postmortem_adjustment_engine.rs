use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal postmortem adjustment calculation.
pub trait SeasonalPostmortemAdjustmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalPostmortemAdjustmentAdjuster {
    pub percent: f64,
}

impl SeasonalPostmortemAdjustmentModifier for PercentSeasonalPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalPostmortemAdjustmentAdjuster {
    pub minimum: f64,
}

impl SeasonalPostmortemAdjustmentModifier for MinimumSeasonalPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalPostmortemAdjustmentAdjuster {
    pub step: f64,
}

impl SeasonalPostmortemAdjustmentModifier for RoundingSeasonalPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalPostmortemAdjustmentAdjuster {
    pub cap: f64,
}

impl SeasonalPostmortemAdjustmentModifier for CapSeasonalPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalPostmortemAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalPostmortemAdjustmentModifier for TieredSeasonalPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalPostmortemAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalPostmortemAdjustmentModifier for FlaggedSeasonalPostmortemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalPostmortemAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the seasonal postmortem adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalPostmortemAdjustmentEngine {
    pub steps: Vec<Rc<dyn SeasonalPostmortemAdjustmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalPostmortemAdjustmentEngine {
    pub fn compute(&mut self, item: &SeasonalPostmortemAdjustment) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalPostmortemAdjustmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalPostmortemAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_postmortem_adjustment_engine(flags: &flags::Registry) -> SeasonalPostmortemAdjustmentEngine {
    SeasonalPostmortemAdjustmentEngine {
        steps: vec![
            Rc::new(PercentSeasonalPostmortemAdjustmentAdjuster { percent: -9.3 })
                as Rc<dyn SeasonalPostmortemAdjustmentModifier>,
            Rc::new(MinimumSeasonalPostmortemAdjustmentAdjuster { minimum: 32.6 }),
            Rc::new(RoundingSeasonalPostmortemAdjustmentAdjuster { step: 0.25 }),
            Rc::new(CapSeasonalPostmortemAdjustmentAdjuster { cap: 48130.0 }),
            Rc::new(TieredSeasonalPostmortemAdjustmentAdjuster {
                bounds: vec![145.0, 869.0],
                rates: vec![1.13, 0.89],
            }),
            Rc::new(FlaggedSeasonalPostmortemAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "seasonal-postmortem-adjustment-legacy".to_string(),
                percent: -7.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 60171.1,
    }
}
