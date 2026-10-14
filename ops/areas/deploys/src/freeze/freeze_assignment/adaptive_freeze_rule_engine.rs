use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the adaptive freeze rule calculation.
pub trait AdaptiveFreezeRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveFreezeRuleAdjuster {
    pub percent: f64,
}

impl AdaptiveFreezeRuleAdjuster for PercentAdaptiveFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveFreezeRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveFreezeRuleAdjuster for TieredAdaptiveFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFreezeRuleEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveFreezeRuleAdjuster for AdaptiveFreezeRuleEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveFreezeRuleAdjuster {
    pub step: f64,
}

impl AdaptiveFreezeRuleAdjuster for RoundingAdaptiveFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveFreezeRuleAdjuster {
    pub minimum: f64,
}

impl AdaptiveFreezeRuleAdjuster for MinimumAdaptiveFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveFreezeRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveFreezeRuleAdjuster for FlaggedAdaptiveFreezeRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveFreezeRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the adaptive freeze rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveFreezeRuleEngine {
    pub steps: Vec<Rc<dyn AdaptiveFreezeRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveFreezeRuleEngine {
    pub fn compute(&mut self, item: &AdaptiveFreezeRule) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveFreezeRuleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveFreezeRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_adaptive_freeze_rule_engine(flags: &flags::Registry) -> AdaptiveFreezeRuleEngine {
    AdaptiveFreezeRuleEngine {
        steps: vec![
            Rc::new(PercentAdaptiveFreezeRuleAdjuster { percent: -15.0 }) as Rc<dyn AdaptiveFreezeRuleAdjuster>,
            Rc::new(TieredAdaptiveFreezeRuleAdjuster {
                bounds: vec![195.0, 4271.0, 5664.0, 6144.0],
                rates: vec![0.89, 0.92, 1.05, 1.01],
            }),
            Rc::new(AdaptiveFreezeRuleEndsAtOffsetAdjuster { weight: 1.4 }),
            Rc::new(RoundingAdaptiveFreezeRuleAdjuster { step: 0.05 }),
            Rc::new(MinimumAdaptiveFreezeRuleAdjuster { minimum: 39.0 }),
            Rc::new(FlaggedAdaptiveFreezeRuleAdjuster {
                flags: flags.clone(),
                flag: "adaptive-freeze-rule-weighted".to_string(),
                percent: 4.4,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 41284.2,
    }
}
