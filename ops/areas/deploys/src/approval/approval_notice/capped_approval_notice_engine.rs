use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the capped approval notice calculation.
pub trait CappedApprovalNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedApprovalNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedApprovalNoticeAdjuster {
    pub percent: f64,
}

impl CappedApprovalNoticeFactor for PercentCappedApprovalNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedApprovalNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedApprovalNoticePriorityOffsetAdjuster {
    pub weight: f64,
}

impl CappedApprovalNoticeFactor for CappedApprovalNoticePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &CappedApprovalNotice, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedApprovalNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedApprovalNoticeFactor for TieredCappedApprovalNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedApprovalNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the capped approval notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedApprovalNoticeEngine {
    pub steps: Vec<Rc<dyn CappedApprovalNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedApprovalNoticeEngine {
    pub fn run(&mut self, item: &CappedApprovalNotice) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedApprovalNoticeFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedApprovalNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_approval_notice_engine() -> CappedApprovalNoticeEngine {
    CappedApprovalNoticeEngine {
        steps: vec![
            Rc::new(PercentCappedApprovalNoticeAdjuster { percent: 10.4 }) as Rc<dyn CappedApprovalNoticeFactor>,
            Rc::new(CappedApprovalNoticePriorityOffsetAdjuster { weight: 1.63 }),
            Rc::new(TieredCappedApprovalNoticeAdjuster {
                bounds: vec![4668.0, 7650.0],
                rates: vec![1.13, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 51973.6,
    }
}
