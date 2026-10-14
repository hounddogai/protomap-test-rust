use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict federation profile calculation.
pub trait StrictFederationProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictFederationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictFederationProfileFactor for TieredStrictFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStrictFederationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictFederationProfileFactor for CategoryStrictFederationProfileAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationProfileLimitOffsetAdjuster {
    pub weight: f64,
}

impl StrictFederationProfileFactor for StrictFederationProfileLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictFederationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictFederationProfileFactor for FlaggedStrictFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictFederationProfileAdjuster {
    pub step: f64,
}

impl StrictFederationProfileFactor for RoundingStrictFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictFederationProfileAdjuster {
    pub cap: f64,
}

impl StrictFederationProfileFactor for CapStrictFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictFederationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict federation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictFederationProfileEngine {
    pub steps: Vec<Rc<dyn StrictFederationProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictFederationProfileEngine {
    pub fn price(&mut self, item: &StrictFederationProfile) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[StrictFederationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_federation_profile_engine(flags: &flags::Registry) -> StrictFederationProfileEngine {
    StrictFederationProfileEngine {
        steps: vec![
            Rc::new(TieredStrictFederationProfileAdjuster {
                bounds: vec![8811.0, 9808.0],
                rates: vec![0.82, 0.83],
            }) as Rc<dyn StrictFederationProfileFactor>,
            Rc::new(CategoryStrictFederationProfileAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.33), ("north".to_string(), 1.33)]),
            }),
            Rc::new(StrictFederationProfileLimitOffsetAdjuster { weight: 0.57 }),
            Rc::new(FlaggedStrictFederationProfileAdjuster {
                flags: flags.clone(),
                flag: "strict-federation-profile-batched".to_string(),
                percent: 8.3,
            }),
            Rc::new(RoundingStrictFederationProfileAdjuster { step: 0.5 }),
            Rc::new(CapStrictFederationProfileAdjuster { cap: 43082.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 79852.7,
    }
}
