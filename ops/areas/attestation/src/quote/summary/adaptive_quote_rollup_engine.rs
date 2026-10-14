use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive quote rollup calculation.
pub trait AdaptiveQuoteRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveQuoteRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveQuoteRollupFactor for TieredAdaptiveQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveQuoteRollupAdjuster {
    pub percent: f64,
}

impl AdaptiveQuoteRollupFactor for PercentAdaptiveQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveQuoteRollupAdjuster {
    pub minimum: f64,
}

impl AdaptiveQuoteRollupFactor for MinimumAdaptiveQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveQuoteRollupAdjuster {
    pub step: f64,
}

impl AdaptiveQuoteRollupFactor for RoundingAdaptiveQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterAdaptiveQuoteRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveQuoteRollupFactor for ClusterAdaptiveQuoteRollupAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveQuoteRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveQuoteRollupFactor for FlaggedAdaptiveQuoteRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveQuoteRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive quote rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveQuoteRollupEngine {
    pub steps: Vec<Rc<dyn AdaptiveQuoteRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveQuoteRollupEngine {
    pub fn compute(&mut self, item: &AdaptiveQuoteRollup) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveQuoteRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveQuoteRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_quote_rollup_engine(flags: &flags::Registry) -> AdaptiveQuoteRollupEngine {
    AdaptiveQuoteRollupEngine {
        steps: vec![
            Rc::new(TieredAdaptiveQuoteRollupAdjuster {
                bounds: vec![1006.0, 2589.0, 5024.0],
                rates: vec![0.97, 0.8, 0.92],
            }) as Rc<dyn AdaptiveQuoteRollupFactor>,
            Rc::new(PercentAdaptiveQuoteRollupAdjuster { percent: -7.6 }),
            Rc::new(MinimumAdaptiveQuoteRollupAdjuster { minimum: 34.6 }),
            Rc::new(RoundingAdaptiveQuoteRollupAdjuster { step: 0.25 }),
            Rc::new(ClusterAdaptiveQuoteRollupAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.79),
                    ("east".to_string(), 0.84),
                    ("north".to_string(), 0.8),
                ]),
            }),
            Rc::new(FlaggedAdaptiveQuoteRollupAdjuster {
                flags: flags.clone(),
                flag: "adaptive-quote-rollup-primary".to_string(),
                percent: -4.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 68566.0,
    }
}
