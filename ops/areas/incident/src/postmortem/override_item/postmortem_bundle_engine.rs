use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the postmortem bundle calculation.
pub trait PostmortemBundleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemBundleStep for PostmortemBundleWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemBundle, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemBundleStep for TieredPostmortemBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPostmortemBundleAdjuster {
    pub minimum: f64,
}

impl PostmortemBundleStep for MinimumPostmortemBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PostmortemBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPostmortemBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PostmortemBundleStep for ChannelPostmortemBundleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PostmortemBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the postmortem bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemBundleEngine {
    pub steps: Vec<Rc<dyn PostmortemBundleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemBundleEngine {
    pub fn price(&mut self, item: &PostmortemBundle) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_bundle_engine() -> PostmortemBundleEngine {
    PostmortemBundleEngine {
        steps: vec![
            Rc::new(PostmortemBundleWindowDaysOffsetAdjuster { weight: 0.12 }) as Rc<dyn PostmortemBundleStep>,
            Rc::new(TieredPostmortemBundleAdjuster {
                bounds: vec![2040.0, 5319.0],
                rates: vec![0.88, 0.8],
            }),
            Rc::new(MinimumPostmortemBundleAdjuster { minimum: 4.9 }),
            Rc::new(ChannelPostmortemBundleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.98)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 48690.7,
    }
}
