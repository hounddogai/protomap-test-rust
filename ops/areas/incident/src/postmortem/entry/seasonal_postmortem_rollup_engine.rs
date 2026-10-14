use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal postmortem rollup calculation.
pub trait SeasonalPostmortemRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalPostmortemRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalPostmortemRollupAdjuster {
    pub percent: f64,
}

impl SeasonalPostmortemRollupFactor for PercentSeasonalPostmortemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalPostmortemRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalPostmortemRollupAdjuster {
    pub cap: f64,
}

impl SeasonalPostmortemRollupFactor for CapSeasonalPostmortemRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalPostmortemRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalPostmortemRollupAdjuster {
    pub step: f64,
}

impl SeasonalPostmortemRollupFactor for RoundingSeasonalPostmortemRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalPostmortemRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalPostmortemRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalPostmortemRollupFactor for TieredSeasonalPostmortemRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalPostmortemRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal postmortem rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalPostmortemRollupEngine {
    pub steps: Vec<Rc<dyn SeasonalPostmortemRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalPostmortemRollupEngine {
    pub fn compute(&mut self, item: &SeasonalPostmortemRollup) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalPostmortemRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalPostmortemRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_postmortem_rollup_engine() -> SeasonalPostmortemRollupEngine {
    SeasonalPostmortemRollupEngine {
        steps: vec![
            Rc::new(PercentSeasonalPostmortemRollupAdjuster { percent: 17.1 })
                as Rc<dyn SeasonalPostmortemRollupFactor>,
            Rc::new(CapSeasonalPostmortemRollupAdjuster { cap: 41965.0 }),
            Rc::new(RoundingSeasonalPostmortemRollupAdjuster { step: 0.1 }),
            Rc::new(TieredSeasonalPostmortemRollupAdjuster {
                bounds: vec![8162.0, 8311.0],
                rates: vec![1.1, 1.0],
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 18670.4,
    }
}
