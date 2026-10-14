use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the query bundle calculation.
pub trait QueryBundleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameQueryBundleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QueryBundleFactor for NameQueryBundleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &QueryBundle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryBundleAdjuster {
    pub step: f64,
}

impl QueryBundleFactor for RoundingQueryBundleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryBundle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryBundleAdjuster {
    pub minimum: f64,
}

impl QueryBundleFactor for MinimumQueryBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryBundleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryBundleFactor for TieredQueryBundleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryBundle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the query bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryBundleEngine {
    pub steps: Vec<Rc<dyn QueryBundleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryBundleEngine {
    pub fn evaluate(&mut self, item: &QueryBundle) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[QueryBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_bundle_engine() -> QueryBundleEngine {
    QueryBundleEngine {
        steps: vec![
            Rc::new(NameQueryBundleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.77), ("north".to_string(), 0.77)]),
            }) as Rc<dyn QueryBundleFactor>,
            Rc::new(RoundingQueryBundleAdjuster { step: 0.5 }),
            Rc::new(MinimumQueryBundleAdjuster { minimum: 43.3 }),
            Rc::new(TieredQueryBundleAdjuster {
                bounds: vec![535.0, 6045.0, 8670.0],
                rates: vec![1.13, 1.15, 0.96],
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 4016.3,
    }
}
