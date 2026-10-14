use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive reservation profile calculation.
pub trait AdaptiveReservationProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveReservationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveReservationProfileAdjuster {
    pub minimum: f64,
}

impl AdaptiveReservationProfileModifier for MinimumAdaptiveReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveReservationProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveReservationProfileAdjuster {
    pub percent: f64,
}

impl AdaptiveReservationProfileModifier for PercentAdaptiveReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveReservationProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveReservationProfileAdjuster {
    pub step: f64,
}

impl AdaptiveReservationProfileModifier for RoundingAdaptiveReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveReservationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveReservationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveReservationProfileModifier for TieredAdaptiveReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveReservationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReservationProfileWeightOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveReservationProfileModifier for AdaptiveReservationProfileWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveReservationProfile, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the adaptive reservation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveReservationProfileEngine {
    pub steps: Vec<Rc<dyn AdaptiveReservationProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveReservationProfileEngine {
    pub fn run(&mut self, item: &AdaptiveReservationProfile) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveReservationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_reservation_profile_engine() -> AdaptiveReservationProfileEngine {
    AdaptiveReservationProfileEngine {
        steps: vec![
            Rc::new(MinimumAdaptiveReservationProfileAdjuster { minimum: 12.7 })
                as Rc<dyn AdaptiveReservationProfileModifier>,
            Rc::new(PercentAdaptiveReservationProfileAdjuster { percent: 20.9 }),
            Rc::new(RoundingAdaptiveReservationProfileAdjuster { step: 0.25 }),
            Rc::new(TieredAdaptiveReservationProfileAdjuster {
                bounds: vec![5927.0, 7888.0],
                rates: vec![0.89, 1.06],
            }),
            Rc::new(AdaptiveReservationProfileWeightOffsetAdjuster { weight: 0.17 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 36345.7,
    }
}
