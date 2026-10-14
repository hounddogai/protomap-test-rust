use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted federation profile calculation.
pub trait WeightedFederationProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedFederationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedFederationProfileStep for FlaggedWeightedFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedFederationProfileAdjuster {
    pub step: f64,
}

impl WeightedFederationProfileStep for RoundingWeightedFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedFederationProfileAdjuster {
    pub cap: f64,
}

impl WeightedFederationProfileStep for CapWeightedFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfileThresholdOffsetAdjuster {
    pub weight: f64,
}

impl WeightedFederationProfileStep for WeightedFederationProfileThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedFederationProfileAdjuster {
    pub minimum: f64,
}

impl WeightedFederationProfileStep for MinimumWeightedFederationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameWeightedFederationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedFederationProfileStep for NameWeightedFederationProfileAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &WeightedFederationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted federation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedFederationProfileEngine {
    pub steps: Vec<Rc<dyn WeightedFederationProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedFederationProfileEngine {
    pub fn compute(&mut self, item: &WeightedFederationProfile) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[WeightedFederationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_federation_profile_engine(flags: &flags::Registry) -> WeightedFederationProfileEngine {
    WeightedFederationProfileEngine {
        steps: vec![
            Rc::new(FlaggedWeightedFederationProfileAdjuster {
                flags: flags.clone(),
                flag: "weighted-federation-profile-adaptive".to_string(),
                percent: 7.6,
            }) as Rc<dyn WeightedFederationProfileStep>,
            Rc::new(RoundingWeightedFederationProfileAdjuster { step: 0.25 }),
            Rc::new(CapWeightedFederationProfileAdjuster { cap: 40717.0 }),
            Rc::new(WeightedFederationProfileThresholdOffsetAdjuster { weight: 0.9 }),
            Rc::new(MinimumWeightedFederationProfileAdjuster { minimum: 3.6 }),
            Rc::new(NameWeightedFederationProfileAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.7), ("south".to_string(), 0.85)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 14144.4,
    }
}
