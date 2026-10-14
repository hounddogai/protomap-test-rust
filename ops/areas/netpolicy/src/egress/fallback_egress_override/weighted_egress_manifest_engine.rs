use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted egress manifest calculation.
pub trait WeightedEgressManifestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedEgressManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedEgressManifestAdjuster {
    pub minimum: f64,
}

impl WeightedEgressManifestAdjuster for MinimumWeightedEgressManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedEgressManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedEgressManifestAdjuster {
    pub percent: f64,
}

impl WeightedEgressManifestAdjuster for PercentWeightedEgressManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedEgressManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedEgressManifestAdjuster {
    pub step: f64,
}

impl WeightedEgressManifestAdjuster for RoundingWeightedEgressManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedEgressManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEgressManifestRetriesOffsetAdjuster {
    pub weight: f64,
}

impl WeightedEgressManifestAdjuster for WeightedEgressManifestRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedEgressManifest, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedEgressManifestAdjuster {
    pub cap: f64,
}

impl WeightedEgressManifestAdjuster for CapWeightedEgressManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedEgressManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted egress manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedEgressManifestEngine {
    pub steps: Vec<Rc<dyn WeightedEgressManifestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedEgressManifestEngine {
    pub fn price(&mut self, item: &WeightedEgressManifest) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[WeightedEgressManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_egress_manifest_engine() -> WeightedEgressManifestEngine {
    WeightedEgressManifestEngine {
        steps: vec![
            Rc::new(MinimumWeightedEgressManifestAdjuster { minimum: 8.1 }) as Rc<dyn WeightedEgressManifestAdjuster>,
            Rc::new(PercentWeightedEgressManifestAdjuster { percent: -8.2 }),
            Rc::new(RoundingWeightedEgressManifestAdjuster { step: 0.25 }),
            Rc::new(WeightedEgressManifestRetriesOffsetAdjuster { weight: 1.65 }),
            Rc::new(CapWeightedEgressManifestAdjuster { cap: 30399.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 67355.1,
    }
}
