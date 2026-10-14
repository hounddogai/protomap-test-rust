use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted redaction preference calculation.
pub trait WeightedRedactionPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRedactionPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedRedactionPreferenceAdjuster {
    pub percent: f64,
}

impl WeightedRedactionPreferenceAdjuster for PercentWeightedRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedRedactionPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedRedactionPreferenceAdjuster {
    pub cap: f64,
}

impl WeightedRedactionPreferenceAdjuster for CapWeightedRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedRedactionPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRedactionPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRedactionPreferenceAdjuster for TieredWeightedRedactionPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRedactionPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted redaction preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRedactionPreferenceEngine {
    pub steps: Vec<Rc<dyn WeightedRedactionPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRedactionPreferenceEngine {
    pub fn compute(&mut self, item: &WeightedRedactionPreference) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[WeightedRedactionPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_weighted_redaction_preference_engine() -> WeightedRedactionPreferenceEngine {
    WeightedRedactionPreferenceEngine {
        steps: vec![
            Rc::new(PercentWeightedRedactionPreferenceAdjuster { percent: 11.2 })
                as Rc<dyn WeightedRedactionPreferenceAdjuster>,
            Rc::new(CapWeightedRedactionPreferenceAdjuster { cap: 12758.0 }),
            Rc::new(TieredWeightedRedactionPreferenceAdjuster {
                bounds: vec![3059.0, 8194.0],
                rates: vec![0.99, 1.2],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 48443.4,
    }
}
