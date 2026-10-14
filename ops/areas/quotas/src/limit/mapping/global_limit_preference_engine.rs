use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the global limit preference calculation.
pub trait GlobalLimitPreferenceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalLimitPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalLimitPreferenceAdjuster {
    pub minimum: f64,
}

impl GlobalLimitPreferenceFactor for MinimumGlobalLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalLimitPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalLimitPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalLimitPreferenceFactor for FlaggedGlobalLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalLimitPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalLimitPreferenceAdjuster {
    pub percent: f64,
}

impl GlobalLimitPreferenceFactor for PercentGlobalLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalLimitPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreferenceWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl GlobalLimitPreferenceFactor for GlobalLimitPreferenceWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalLimitPreference, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalLimitPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalLimitPreferenceFactor for TieredGlobalLimitPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalLimitPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the global limit preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalLimitPreferenceEngine {
    pub steps: Vec<Rc<dyn GlobalLimitPreferenceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalLimitPreferenceEngine {
    pub fn price(&mut self, item: &GlobalLimitPreference) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalLimitPreferenceFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalLimitPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_limit_preference_engine(flags: &flags::Registry) -> GlobalLimitPreferenceEngine {
    GlobalLimitPreferenceEngine {
        steps: vec![
            Rc::new(MinimumGlobalLimitPreferenceAdjuster { minimum: 32.8 }) as Rc<dyn GlobalLimitPreferenceFactor>,
            Rc::new(FlaggedGlobalLimitPreferenceAdjuster {
                flags: flags.clone(),
                flag: "global-limit-preference-strict".to_string(),
                percent: 5.2,
            }),
            Rc::new(PercentGlobalLimitPreferenceAdjuster { percent: -7.6 }),
            Rc::new(GlobalLimitPreferenceWindowDaysOffsetAdjuster { weight: 1.59 }),
            Rc::new(TieredGlobalLimitPreferenceAdjuster {
                bounds: vec![1133.0, 3501.0, 4538.0, 8380.0],
                rates: vec![1.0, 1.15, 1.03, 1.02],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 39888.3,
    }
}
