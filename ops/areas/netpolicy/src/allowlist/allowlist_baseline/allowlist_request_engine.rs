use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the allowlist request calculation.
pub trait AllowlistRequestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAllowlistRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AllowlistRequestFactor for TieredAllowlistRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AllowlistRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAllowlistRequestAdjuster {
    pub percent: f64,
}

impl AllowlistRequestFactor for PercentAllowlistRequestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AllowlistRequest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAllowlistRequestAdjuster {
    pub minimum: f64,
}

impl AllowlistRequestFactor for MinimumAllowlistRequestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AllowlistRequest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRequestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistRequestFactor for AllowlistRequestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistRequest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the allowlist request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistRequestEngine {
    pub steps: Vec<Rc<dyn AllowlistRequestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistRequestEngine {
    pub fn run(&mut self, item: &AllowlistRequest) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AllowlistRequestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_allowlist_request_engine() -> AllowlistRequestEngine {
    AllowlistRequestEngine {
        steps: vec![
            Rc::new(TieredAllowlistRequestAdjuster {
                bounds: vec![3836.0, 5698.0, 8653.0],
                rates: vec![1.15, 1.02, 0.81],
            }) as Rc<dyn AllowlistRequestFactor>,
            Rc::new(PercentAllowlistRequestAdjuster { percent: -6.9 }),
            Rc::new(MinimumAllowlistRequestAdjuster { minimum: 36.7 }),
            Rc::new(AllowlistRequestCapacityOffsetAdjuster { weight: 1.96 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 81208.4,
    }
}
