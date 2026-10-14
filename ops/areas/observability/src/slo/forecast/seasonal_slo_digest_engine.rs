use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal slo digest calculation.
pub trait SeasonalSloDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalSloDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalSloDigestAdjuster {
    pub cap: f64,
}

impl SeasonalSloDigestAdjuster for CapSeasonalSloDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalSloDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalSloDigestAdjuster {
    pub step: f64,
}

impl SeasonalSloDigestAdjuster for RoundingSeasonalSloDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalSloDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalSloDigestAdjuster {
    pub percent: f64,
}

impl SeasonalSloDigestAdjuster for PercentSeasonalSloDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalSloDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalSloDigestAdjuster {
    pub minimum: f64,
}

impl SeasonalSloDigestAdjuster for MinimumSeasonalSloDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalSloDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the seasonal slo digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalSloDigestEngine {
    pub steps: Vec<Rc<dyn SeasonalSloDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalSloDigestEngine {
    pub fn price(&mut self, item: &SeasonalSloDigest) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalSloDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalSloDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_slo_digest_engine() -> SeasonalSloDigestEngine {
    SeasonalSloDigestEngine {
        steps: vec![
            Rc::new(CapSeasonalSloDigestAdjuster { cap: 45306.0 }) as Rc<dyn SeasonalSloDigestAdjuster>,
            Rc::new(RoundingSeasonalSloDigestAdjuster { step: 0.05 }),
            Rc::new(PercentSeasonalSloDigestAdjuster { percent: 21.5 }),
            Rc::new(MinimumSeasonalSloDigestAdjuster { minimum: 43.3 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 70698.9,
    }
}
