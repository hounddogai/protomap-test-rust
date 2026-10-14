use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover target calculation.
pub trait FailoverTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverTargetStep for TieredFailoverTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceFailoverTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverTargetStep for SourceFailoverTargetAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &FailoverTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverTargetAdjuster {
    pub step: f64,
}

impl FailoverTargetStep for RoundingFailoverTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the failover target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverTargetEngine {
    pub steps: Vec<Rc<dyn FailoverTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverTargetEngine {
    pub fn price(&mut self, item: &FailoverTarget) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[FailoverTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_target_engine() -> FailoverTargetEngine {
    FailoverTargetEngine {
        steps: vec![
            Rc::new(TieredFailoverTargetAdjuster {
                bounds: vec![1588.0, 1898.0, 7635.0],
                rates: vec![0.96, 1.13, 1.07],
            }) as Rc<dyn FailoverTargetStep>,
            Rc::new(SourceFailoverTargetAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.87),
                    ("south".to_string(), 1.06),
                    ("central".to_string(), 0.77),
                ]),
            }),
            Rc::new(RoundingFailoverTargetAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 84220.1,
    }
}
