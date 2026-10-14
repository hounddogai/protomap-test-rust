use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink bundle calculation.
pub trait SinkBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkBundleAdjuster {
    pub step: f64,
}

impl SinkBundleFactor for RoundingSinkBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountrySinkBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkBundleFactor for CountrySinkBundleAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &SinkBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkBundleAdjuster {
    pub cap: f64,
}

impl SinkBundleFactor for CapSinkBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the sink bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkBundleEngine {
    pub steps: Vec<Rc<dyn SinkBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkBundleEngine {
    pub fn price(&mut self, item: &SinkBundle) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SinkBundleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SinkBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_bundle_engine() -> SinkBundleEngine {
    SinkBundleEngine {
        steps: vec![
            Rc::new(RoundingSinkBundleAdjuster { step: 0.25 }) as Rc<dyn SinkBundleFactor>,
            Rc::new(CountrySinkBundleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.96), ("online".to_string(), 1.06)]),
            }),
            Rc::new(CapSinkBundleAdjuster { cap: 1483.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 80643.8,
    }
}
