use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant notice calculation.
pub trait GrantNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantNoticeAdjuster {
    pub cap: f64,
}

impl GrantNoticeStep for CapGrantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantNoticeAdjuster {
    pub percent: f64,
}

impl GrantNoticeStep for PercentGrantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantNoticeCeilingOffsetAdjuster {
    pub weight: f64,
}

impl GrantNoticeStep for GrantNoticeCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &GrantNotice, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantNoticeAdjuster {
    pub step: f64,
}

impl GrantNoticeStep for RoundingGrantNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the grant notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantNoticeEngine {
    pub steps: Vec<Rc<dyn GrantNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantNoticeEngine {
    pub fn run(&mut self, item: &GrantNotice) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GrantNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_grant_notice_engine() -> GrantNoticeEngine {
    GrantNoticeEngine {
        steps: vec![
            Rc::new(CapGrantNoticeAdjuster { cap: 44939.0 }) as Rc<dyn GrantNoticeStep>,
            Rc::new(PercentGrantNoticeAdjuster { percent: 22.9 }),
            Rc::new(GrantNoticeCeilingOffsetAdjuster { weight: 1.76 }),
            Rc::new(RoundingGrantNoticeAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 19304.2,
    }
}
