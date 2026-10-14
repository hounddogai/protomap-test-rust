use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the isolation notice calculation.
pub trait IsolationNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIsolationNoticeAdjuster {
    pub cap: f64,
}

impl IsolationNoticeStep for CapIsolationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationNoticeAdjuster {
    pub percent: f64,
}

impl IsolationNoticeStep for PercentIsolationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameIsolationNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationNoticeStep for NameIsolationNoticeAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationNoticeDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl IsolationNoticeStep for IsolationNoticeDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationNoticeAdjuster {
    pub minimum: f64,
}

impl IsolationNoticeStep for MinimumIsolationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationNoticeAdjuster {
    pub step: f64,
}

impl IsolationNoticeStep for RoundingIsolationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the isolation notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationNoticeEngine {
    pub steps: Vec<Rc<dyn IsolationNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationNoticeEngine {
    pub fn price(&mut self, item: &IsolationNotice) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[IsolationNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_notice_engine() -> IsolationNoticeEngine {
    IsolationNoticeEngine {
        steps: vec![
            Rc::new(CapIsolationNoticeAdjuster { cap: 28133.0 }) as Rc<dyn IsolationNoticeStep>,
            Rc::new(PercentIsolationNoticeAdjuster { percent: -24.9 }),
            Rc::new(NameIsolationNoticeAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.29)]),
            }),
            Rc::new(IsolationNoticeDurationMinutesOffsetAdjuster { weight: 1.6 }),
            Rc::new(MinimumIsolationNoticeAdjuster { minimum: 21.6 }),
            Rc::new(RoundingIsolationNoticeAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 13555.9,
    }
}
