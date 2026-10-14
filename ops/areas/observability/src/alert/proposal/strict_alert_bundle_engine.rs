use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict alert bundle calculation.
pub trait StrictAlertBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAlertBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameStrictAlertBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictAlertBundleModifier for NameStrictAlertBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &StrictAlertBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictAlertBundleAdjuster {
    pub step: f64,
}

impl StrictAlertBundleModifier for RoundingStrictAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictAlertBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictAlertBundleAdjuster {
    pub cap: f64,
}

impl StrictAlertBundleModifier for CapStrictAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictAlertBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAlertBundleStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictAlertBundleModifier for StrictAlertBundleStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictAlertBundle, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAlertBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAlertBundleModifier for TieredStrictAlertBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAlertBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the strict alert bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAlertBundleEngine {
    pub steps: Vec<Rc<dyn StrictAlertBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAlertBundleEngine {
    pub fn price(&mut self, item: &StrictAlertBundle) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictAlertBundleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictAlertBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_alert_bundle_engine() -> StrictAlertBundleEngine {
    StrictAlertBundleEngine {
        steps: vec![
            Rc::new(NameStrictAlertBundleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.72),
                    ("retail".to_string(), 1.02),
                    ("north".to_string(), 1.22),
                ]),
            }) as Rc<dyn StrictAlertBundleModifier>,
            Rc::new(RoundingStrictAlertBundleAdjuster { step: 1.0 }),
            Rc::new(CapStrictAlertBundleAdjuster { cap: 23741.0 }),
            Rc::new(StrictAlertBundleStartsAtOffsetAdjuster { weight: 1.88 }),
            Rc::new(TieredStrictAlertBundleAdjuster {
                bounds: vec![3327.0, 5004.0],
                rates: vec![1.17, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 79052.8,
    }
}
