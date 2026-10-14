use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the regional binding batch calculation.
pub trait RegionalBindingBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalBindingBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalBindingBatchAdjuster {
    pub percent: f64,
}

impl RegionalBindingBatchModifier for PercentRegionalBindingBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalBindingBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalBindingBatchAdjuster {
    pub minimum: f64,
}

impl RegionalBindingBatchModifier for MinimumRegionalBindingBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalBindingBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRegionalBindingBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalBindingBatchModifier for RegionRegionalBindingBatchAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RegionalBindingBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional binding batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalBindingBatchEngine {
    pub steps: Vec<Rc<dyn RegionalBindingBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalBindingBatchEngine {
    pub fn run(&mut self, item: &RegionalBindingBatch) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalBindingBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalBindingBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_binding_batch_engine() -> RegionalBindingBatchEngine {
    RegionalBindingBatchEngine {
        steps: vec![
            Rc::new(PercentRegionalBindingBatchAdjuster { percent: -10.6 }) as Rc<dyn RegionalBindingBatchModifier>,
            Rc::new(MinimumRegionalBindingBatchAdjuster { minimum: 3.9 }),
            Rc::new(RegionRegionalBindingBatchAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.17), ("east".to_string(), 1.16)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 10467.8,
    }
}
