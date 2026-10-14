use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict egress preference calculation.
pub trait StrictEgressPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryStrictEgressPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictEgressPreferenceModifier for CountryStrictEgressPreferenceAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictEgressPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictEgressPreferenceModifier for TieredStrictEgressPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictEgressPreferenceAdjuster {
    pub percent: f64,
}

impl StrictEgressPreferenceModifier for PercentStrictEgressPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressPreferenceRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl StrictEgressPreferenceModifier for StrictEgressPreferenceRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictEgressPreferenceAdjuster {
    pub cap: f64,
}

impl StrictEgressPreferenceModifier for CapStrictEgressPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictEgressPreferenceAdjuster {
    pub step: f64,
}

impl StrictEgressPreferenceModifier for RoundingStrictEgressPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictEgressPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the strict egress preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictEgressPreferenceEngine {
    pub steps: Vec<Rc<dyn StrictEgressPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictEgressPreferenceEngine {
    pub fn compute(&mut self, item: &StrictEgressPreference) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictEgressPreferenceModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictEgressPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_egress_preference_engine() -> StrictEgressPreferenceEngine {
    StrictEgressPreferenceEngine {
        steps: vec![
            Rc::new(CountryStrictEgressPreferenceAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.8)]),
            }) as Rc<dyn StrictEgressPreferenceModifier>,
            Rc::new(TieredStrictEgressPreferenceAdjuster {
                bounds: vec![352.0, 8495.0, 8881.0],
                rates: vec![1.02, 1.11, 1.16],
            }),
            Rc::new(PercentStrictEgressPreferenceAdjuster { percent: 22.0 }),
            Rc::new(StrictEgressPreferenceRotationDaysOffsetAdjuster { weight: 0.75 }),
            Rc::new(CapStrictEgressPreferenceAdjuster { cap: 46973.0 }),
            Rc::new(RoundingStrictEgressPreferenceAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 97415.9,
    }
}
