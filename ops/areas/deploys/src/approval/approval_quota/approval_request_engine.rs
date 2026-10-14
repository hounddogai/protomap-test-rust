use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval request calculation.
pub trait ApprovalRequestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRequestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl ApprovalRequestStep for ApprovalRequestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &ApprovalRequest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalRequestAdjuster {
    pub step: f64,
}

impl ApprovalRequestStep for RoundingApprovalRequestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalRequest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalRequestStep for TieredApprovalRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the approval request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalRequestEngine {
    pub steps: Vec<Rc<dyn ApprovalRequestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalRequestEngine {
    pub fn price(&mut self, item: &ApprovalRequest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ApprovalRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_request_engine() -> ApprovalRequestEngine {
    ApprovalRequestEngine {
        steps: vec![
            Rc::new(ApprovalRequestCapacityOffsetAdjuster { weight: 0.04 }) as Rc<dyn ApprovalRequestStep>,
            Rc::new(RoundingApprovalRequestAdjuster { step: 0.1 }),
            Rc::new(TieredApprovalRequestAdjuster {
                bounds: vec![4547.0, 8322.0, 8511.0, 9395.0],
                rates: vec![0.93, 0.9, 0.87, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 23149.3,
    }
}
