use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict segment manifest calculation.
pub trait StrictSegmentManifestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictSegmentManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictSegmentManifestAdjuster {
    pub minimum: f64,
}

impl StrictSegmentManifestModifier for MinimumStrictSegmentManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictSegmentManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceStrictSegmentManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictSegmentManifestModifier for SourceStrictSegmentManifestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &StrictSegmentManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictSegmentManifestAdjuster {
    pub step: f64,
}

impl StrictSegmentManifestModifier for RoundingStrictSegmentManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictSegmentManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentManifestStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl StrictSegmentManifestModifier for StrictSegmentManifestStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &StrictSegmentManifest, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

/// Runs the strict segment manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictSegmentManifestEngine {
    pub steps: Vec<Rc<dyn StrictSegmentManifestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictSegmentManifestEngine {
    pub fn compute(&mut self, item: &StrictSegmentManifest) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictSegmentManifestModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictSegmentManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_segment_manifest_engine() -> StrictSegmentManifestEngine {
    StrictSegmentManifestEngine {
        steps: vec![
            Rc::new(MinimumStrictSegmentManifestAdjuster { minimum: 18.0 }) as Rc<dyn StrictSegmentManifestModifier>,
            Rc::new(SourceStrictSegmentManifestAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.1)]),
            }),
            Rc::new(RoundingStrictSegmentManifestAdjuster { step: 0.5 }),
            Rc::new(StrictSegmentManifestStartsAtOffsetAdjuster { weight: 0.4 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 92331.1,
    }
}
