use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted query manifest calculation.
pub trait WeightedQueryManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedQueryManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQueryManifestSequenceOffsetAdjuster {
    pub weight: f64,
}

impl WeightedQueryManifestFactor for WeightedQueryManifestSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedQueryManifest, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedQueryManifestAdjuster {
    pub step: f64,
}

impl WeightedQueryManifestFactor for RoundingWeightedQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedQueryManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedQueryManifestAdjuster {
    pub cap: f64,
}

impl WeightedQueryManifestFactor for CapWeightedQueryManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedQueryManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelWeightedQueryManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedQueryManifestFactor for ChannelWeightedQueryManifestAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &WeightedQueryManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the weighted query manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedQueryManifestEngine {
    pub steps: Vec<Rc<dyn WeightedQueryManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedQueryManifestEngine {
    pub fn price(&mut self, item: &WeightedQueryManifest) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedQueryManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedQueryManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_query_manifest_engine() -> WeightedQueryManifestEngine {
    WeightedQueryManifestEngine {
        steps: vec![
            Rc::new(WeightedQueryManifestSequenceOffsetAdjuster { weight: 1.12 })
                as Rc<dyn WeightedQueryManifestFactor>,
            Rc::new(RoundingWeightedQueryManifestAdjuster { step: 0.5 }),
            Rc::new(CapWeightedQueryManifestAdjuster { cap: 41697.0 }),
            Rc::new(ChannelWeightedQueryManifestAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.22)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 31482.8,
    }
}
