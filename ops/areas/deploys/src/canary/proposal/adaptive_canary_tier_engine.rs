use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive canary tier calculation.
pub trait AdaptiveCanaryTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveCanaryTierAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveCanaryTierStep for AdaptiveCanaryTierAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveCanaryTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveCanaryTierStep for FlaggedAdaptiveCanaryTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelAdaptiveCanaryTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveCanaryTierStep for LabelAdaptiveCanaryTierAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptiveCanaryTierAdjuster {
    pub cap: f64,
}

impl AdaptiveCanaryTierStep for CapAdaptiveCanaryTierAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveCanaryTierAdjuster {
    pub percent: f64,
}

impl AdaptiveCanaryTierStep for PercentAdaptiveCanaryTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveCanaryTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveCanaryTierStep for TieredAdaptiveCanaryTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveCanaryTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the adaptive canary tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveCanaryTierEngine {
    pub steps: Vec<Rc<dyn AdaptiveCanaryTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveCanaryTierEngine {
    pub fn run(&mut self, item: &AdaptiveCanaryTier) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveCanaryTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveCanaryTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_canary_tier_engine(flags: &flags::Registry) -> AdaptiveCanaryTierEngine {
    AdaptiveCanaryTierEngine {
        steps: vec![
            Rc::new(AdaptiveCanaryTierAttemptsOffsetAdjuster { weight: 1.38 }) as Rc<dyn AdaptiveCanaryTierStep>,
            Rc::new(FlaggedAdaptiveCanaryTierAdjuster {
                flags: flags.clone(),
                flag: "adaptive-canary-tier-primary".to_string(),
                percent: 0.7,
            }),
            Rc::new(LabelAdaptiveCanaryTierAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.91),
                    ("retail".to_string(), 1.38),
                    ("north".to_string(), 1.25),
                ]),
            }),
            Rc::new(CapAdaptiveCanaryTierAdjuster { cap: 2650.0 }),
            Rc::new(PercentAdaptiveCanaryTierAdjuster { percent: -19.4 }),
            Rc::new(TieredAdaptiveCanaryTierAdjuster {
                bounds: vec![1261.0, 3810.0],
                rates: vec![0.92, 1.02],
            }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 64783.3,
    }
}
