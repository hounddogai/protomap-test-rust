use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence profile calculation.
pub trait SilenceProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategorySilenceProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceProfileAdjuster for CategorySilenceProfileAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSilenceProfileAdjuster {
    pub cap: f64,
}

impl SilenceProfileAdjuster for CapSilenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceProfileAdjuster {
    pub minimum: f64,
}

impl SilenceProfileAdjuster for MinimumSilenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceProfileAdjuster {
    pub step: f64,
}

impl SilenceProfileAdjuster for RoundingSilenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceProfileAdjuster for TieredSilenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceProfileAdjuster {
    pub percent: f64,
}

impl SilenceProfileAdjuster for PercentSilenceProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the silence profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceProfileEngine {
    pub steps: Vec<Rc<dyn SilenceProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceProfileEngine {
    pub fn price(&mut self, item: &SilenceProfile) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[SilenceProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_silence_profile_engine() -> SilenceProfileEngine {
    SilenceProfileEngine {
        steps: vec![
            Rc::new(CategorySilenceProfileAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.3), ("central".to_string(), 0.8)]),
            }) as Rc<dyn SilenceProfileAdjuster>,
            Rc::new(CapSilenceProfileAdjuster { cap: 25582.0 }),
            Rc::new(MinimumSilenceProfileAdjuster { minimum: 31.2 }),
            Rc::new(RoundingSilenceProfileAdjuster { step: 5.0 }),
            Rc::new(TieredSilenceProfileAdjuster {
                bounds: vec![55.0, 3526.0, 6991.0],
                rates: vec![1.18, 0.92, 0.83],
            }),
            Rc::new(PercentSilenceProfileAdjuster { percent: 19.2 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 85273.8,
    }
}
