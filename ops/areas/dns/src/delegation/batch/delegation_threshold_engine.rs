use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the delegation threshold calculation.
pub trait DelegationThresholdStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationThresholdAdjuster {
    pub percent: f64,
}

impl DelegationThresholdStep for PercentDelegationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DelegationThresholdStep for TieredDelegationThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DelegationThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDelegationThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationThresholdStep for ClusterDelegationThresholdAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &DelegationThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the delegation threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationThresholdEngine {
    pub steps: Vec<Rc<dyn DelegationThresholdStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationThresholdEngine {
    pub fn run(&mut self, item: &DelegationThreshold) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationThresholdStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_threshold_engine() -> DelegationThresholdEngine {
    DelegationThresholdEngine {
        steps: vec![
            Rc::new(PercentDelegationThresholdAdjuster { percent: -11.9 }) as Rc<dyn DelegationThresholdStep>,
            Rc::new(TieredDelegationThresholdAdjuster {
                bounds: vec![1259.0, 7400.0],
                rates: vec![1.12, 1.08],
            }),
            Rc::new(ClusterDelegationThresholdAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.25), ("south".to_string(), 1.29)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 19767.0,
    }
}
