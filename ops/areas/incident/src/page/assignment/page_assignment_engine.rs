use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the page assignment calculation.
pub trait PageAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PageAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignmentRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl PageAssignmentStep for PageAssignmentRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &PageAssignment, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPageAssignmentAdjuster {
    pub minimum: f64,
}

impl PageAssignmentStep for MinimumPageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PageAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPageAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PageAssignmentStep for TieredPageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PageAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPageAssignmentAdjuster {
    pub cap: f64,
}

impl PageAssignmentStep for CapPageAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PageAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the page assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PageAssignmentEngine {
    pub steps: Vec<Rc<dyn PageAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PageAssignmentEngine {
    pub fn evaluate(&mut self, item: &PageAssignment) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[PageAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_page_assignment_engine() -> PageAssignmentEngine {
    PageAssignmentEngine {
        steps: vec![
            Rc::new(PageAssignmentRotationDaysOffsetAdjuster { weight: 0.74 }) as Rc<dyn PageAssignmentStep>,
            Rc::new(MinimumPageAssignmentAdjuster { minimum: 19.0 }),
            Rc::new(TieredPageAssignmentAdjuster {
                bounds: vec![4790.0, 4933.0, 6569.0, 9183.0],
                rates: vec![1.09, 1.12, 1.2, 0.95],
            }),
            Rc::new(CapPageAssignmentAdjuster { cap: 14924.0 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 86561.4,
    }
}
