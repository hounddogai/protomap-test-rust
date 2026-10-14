use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce request calculation.
pub trait NonceRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequestQuantityOffsetAdjuster {
    pub weight: f64,
}

impl NonceRequestStep for NonceRequestQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &NonceRequest, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceRequestAdjuster {
    pub cap: f64,
}

impl NonceRequestStep for CapNonceRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredNonceRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl NonceRequestStep for TieredNonceRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &NonceRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelNonceRequestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl NonceRequestStep for LabelNonceRequestAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &NonceRequest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the nonce request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceRequestEngine {
    pub steps: Vec<Rc<dyn NonceRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceRequestEngine {
    pub fn evaluate(&mut self, item: &NonceRequest) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[NonceRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_request_engine() -> NonceRequestEngine {
    NonceRequestEngine {
        steps: vec![
            Rc::new(NonceRequestQuantityOffsetAdjuster { weight: 1.26 }) as Rc<dyn NonceRequestStep>,
            Rc::new(CapNonceRequestAdjuster { cap: 7047.0 }),
            Rc::new(TieredNonceRequestAdjuster {
                bounds: vec![1841.0, 7916.0, 8518.0, 8758.0],
                rates: vec![1.08, 0.81, 0.82, 0.81],
            }),
            Rc::new(LabelNonceRequestAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.06), ("south".to_string(), 1.0)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 41534.3,
    }
}
