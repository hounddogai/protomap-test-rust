use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal identity profile calculation.
pub trait SeasonalIdentityProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalIdentityProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalIdentityProfileAdjuster {
    pub step: f64,
}

impl SeasonalIdentityProfileModifier for RoundingSeasonalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalIdentityProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalIdentityProfileAdjuster {
    pub cap: f64,
}

impl SeasonalIdentityProfileModifier for CapSeasonalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalIdentityProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalIdentityProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalIdentityProfileModifier for TieredSeasonalIdentityProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalIdentityProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the seasonal identity profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalIdentityProfileEngine {
    pub steps: Vec<Rc<dyn SeasonalIdentityProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalIdentityProfileEngine {
    pub fn run(&mut self, item: &SeasonalIdentityProfile) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalIdentityProfileModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalIdentityProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_identity_profile_engine() -> SeasonalIdentityProfileEngine {
    SeasonalIdentityProfileEngine {
        steps: vec![
            Rc::new(RoundingSeasonalIdentityProfileAdjuster { step: 0.5 }) as Rc<dyn SeasonalIdentityProfileModifier>,
            Rc::new(CapSeasonalIdentityProfileAdjuster { cap: 42097.0 }),
            Rc::new(TieredSeasonalIdentityProfileAdjuster {
                bounds: vec![1305.0, 5792.0],
                rates: vec![1.16, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 86668.9,
    }
}
