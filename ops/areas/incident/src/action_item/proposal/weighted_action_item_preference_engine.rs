use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted action item preference calculation.
pub trait WeightedActionItemPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedActionItemPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedActionItemPreferenceAdjuster {
    pub cap: f64,
}

impl WeightedActionItemPreferenceAdjuster for CapWeightedActionItemPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedActionItemPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedActionItemPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedActionItemPreferenceAdjuster for FlaggedWeightedActionItemPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedActionItemPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedActionItemPreferenceAdjuster {
    pub minimum: f64,
}

impl WeightedActionItemPreferenceAdjuster for MinimumWeightedActionItemPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedActionItemPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedActionItemPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedActionItemPreferenceAdjuster for TieredWeightedActionItemPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedActionItemPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedActionItemPreferenceAdjuster {
    pub step: f64,
}

impl WeightedActionItemPreferenceAdjuster for RoundingWeightedActionItemPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedActionItemPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the weighted action item preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedActionItemPreferenceEngine {
    pub steps: Vec<Rc<dyn WeightedActionItemPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedActionItemPreferenceEngine {
    pub fn evaluate(&mut self, item: &WeightedActionItemPreference) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedActionItemPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_weighted_action_item_preference_engine(flags: &flags::Registry) -> WeightedActionItemPreferenceEngine {
    WeightedActionItemPreferenceEngine {
        steps: vec![
            Rc::new(CapWeightedActionItemPreferenceAdjuster { cap: 37907.0 })
                as Rc<dyn WeightedActionItemPreferenceAdjuster>,
            Rc::new(FlaggedWeightedActionItemPreferenceAdjuster {
                flags: flags.clone(),
                flag: "weighted-action-item-preference-adaptive".to_string(),
                percent: -7.9,
            }),
            Rc::new(MinimumWeightedActionItemPreferenceAdjuster { minimum: 1.0 }),
            Rc::new(TieredWeightedActionItemPreferenceAdjuster {
                bounds: vec![785.0, 8822.0, 9074.0],
                rates: vec![0.82, 0.83, 0.91],
            }),
            Rc::new(RoundingWeightedActionItemPreferenceAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 20569.0,
    }
}
