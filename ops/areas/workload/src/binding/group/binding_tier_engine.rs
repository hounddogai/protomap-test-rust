use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the binding tier calculation.
pub trait BindingTierAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingTierAdjuster {
    pub percent: f64,
}

impl BindingTierAdjuster for PercentBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingTier, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingTierAdjuster {
    pub minimum: f64,
}

impl BindingTierAdjuster for MinimumBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBindingTierAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingTierAdjuster for SegmentBindingTierAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &BindingTier, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBindingTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BindingTierAdjuster for FlaggedBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BindingTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingTierAdjuster {
    pub step: f64,
}

impl BindingTierAdjuster for RoundingBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingTierAdjuster for TieredBindingTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the binding tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingTierEngine {
    pub steps: Vec<Rc<dyn BindingTierAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingTierEngine {
    pub fn evaluate(&mut self, item: &BindingTier) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingTierAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_tier_engine(flags: &flags::Registry) -> BindingTierEngine {
    BindingTierEngine {
        steps: vec![
            Rc::new(PercentBindingTierAdjuster { percent: 12.2 }) as Rc<dyn BindingTierAdjuster>,
            Rc::new(MinimumBindingTierAdjuster { minimum: 27.3 }),
            Rc::new(SegmentBindingTierAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.38)]),
            }),
            Rc::new(FlaggedBindingTierAdjuster {
                flags: flags.clone(),
                flag: "binding-tier-regional".to_string(),
                percent: -4.2,
            }),
            Rc::new(RoundingBindingTierAdjuster { step: 5.0 }),
            Rc::new(TieredBindingTierAdjuster {
                bounds: vec![1288.0, 1702.0, 2887.0, 5094.0],
                rates: vec![1.06, 0.92, 0.81, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 76672.8,
    }
}
