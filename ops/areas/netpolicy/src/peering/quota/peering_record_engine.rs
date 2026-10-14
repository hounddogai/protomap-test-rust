use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the peering record calculation.
pub trait PeeringRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PeeringRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPeeringRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PeeringRecordStep for CategoryPeeringRecordAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PeeringRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPeeringRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PeeringRecordStep for TieredPeeringRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PeeringRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRecordAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl PeeringRecordStep for PeeringRecordAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &PeeringRecord, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the peering record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PeeringRecordEngine {
    pub steps: Vec<Rc<dyn PeeringRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PeeringRecordEngine {
    pub fn compute(&mut self, item: &PeeringRecord) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PeeringRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PeeringRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_peering_record_engine() -> PeeringRecordEngine {
    PeeringRecordEngine {
        steps: vec![
            Rc::new(CategoryPeeringRecordAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.39), ("north".to_string(), 0.85)]),
            }) as Rc<dyn PeeringRecordStep>,
            Rc::new(TieredPeeringRecordAdjuster {
                bounds: vec![4142.0, 6575.0, 8662.0],
                rates: vec![1.05, 0.94, 0.8],
            }),
            Rc::new(PeeringRecordAttemptsOffsetAdjuster { weight: 0.57 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 57497.0,
    }
}
