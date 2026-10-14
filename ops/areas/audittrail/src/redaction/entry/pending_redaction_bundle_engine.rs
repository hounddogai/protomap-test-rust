use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending redaction bundle calculation.
pub trait PendingRedactionBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingRedactionBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingRedactionBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingRedactionBundleStep for TieredPendingRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingRedactionBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingRedactionBundleAdjuster {
    pub cap: f64,
}

impl PendingRedactionBundleStep for CapPendingRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingRedactionBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPendingRedactionBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingRedactionBundleStep for ClusterPendingRedactionBundleAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PendingRedactionBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingRedactionBundleAdjuster {
    pub minimum: f64,
}

impl PendingRedactionBundleStep for MinimumPendingRedactionBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingRedactionBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending redaction bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingRedactionBundleEngine {
    pub steps: Vec<Rc<dyn PendingRedactionBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingRedactionBundleEngine {
    pub fn run(&mut self, item: &PendingRedactionBundle) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[PendingRedactionBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_redaction_bundle_engine() -> PendingRedactionBundleEngine {
    PendingRedactionBundleEngine {
        steps: vec![
            Rc::new(TieredPendingRedactionBundleAdjuster {
                bounds: vec![1132.0, 2691.0],
                rates: vec![0.88, 0.84],
            }) as Rc<dyn PendingRedactionBundleStep>,
            Rc::new(CapPendingRedactionBundleAdjuster { cap: 38400.0 }),
            Rc::new(ClusterPendingRedactionBundleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.01)]),
            }),
            Rc::new(MinimumPendingRedactionBundleAdjuster { minimum: 11.5 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 29986.7,
    }
}
