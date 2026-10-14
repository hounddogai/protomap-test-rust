use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the isolation summary calculation.
pub trait IsolationSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationSummaryAdjuster {
    pub minimum: f64,
}

impl IsolationSummaryFactor for MinimumIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationSummaryAdjuster {
    pub cap: f64,
}

impl IsolationSummaryFactor for CapIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationSummaryFactor for TieredIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterIsolationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationSummaryFactor for ClusterIsolationSummaryAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &IsolationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the isolation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationSummaryEngine {
    pub steps: Vec<Rc<dyn IsolationSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationSummaryEngine {
    pub fn compute(&mut self, item: &IsolationSummary) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[IsolationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_isolation_summary_engine() -> IsolationSummaryEngine {
    IsolationSummaryEngine {
        steps: vec![
            Rc::new(MinimumIsolationSummaryAdjuster { minimum: 35.8 }) as Rc<dyn IsolationSummaryFactor>,
            Rc::new(CapIsolationSummaryAdjuster { cap: 16378.0 }),
            Rc::new(TieredIsolationSummaryAdjuster {
                bounds: vec![1023.0, 6350.0, 6405.0, 9287.0],
                rates: vec![0.9, 0.96, 0.87, 0.89],
            }),
            Rc::new(ClusterIsolationSummaryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.8), ("south".to_string(), 1.33)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 51717.1,
    }
}
