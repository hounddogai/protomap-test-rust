use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped retention notice calculation.
pub trait CappedRetentionNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedRetentionNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionNoticeAmountOffsetAdjuster {
    pub weight: f64,
}

impl CappedRetentionNoticeStep for CappedRetentionNoticeAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &CappedRetentionNotice, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedRetentionNoticeAdjuster {
    pub step: f64,
}

impl CappedRetentionNoticeStep for RoundingCappedRetentionNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedRetentionNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedRetentionNoticeAdjuster {
    pub percent: f64,
}

impl CappedRetentionNoticeStep for PercentCappedRetentionNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedRetentionNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedRetentionNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedRetentionNoticeStep for FlaggedCappedRetentionNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedRetentionNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped retention notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedRetentionNoticeEngine {
    pub steps: Vec<Rc<dyn CappedRetentionNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedRetentionNoticeEngine {
    pub fn compute(&mut self, item: &CappedRetentionNotice) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedRetentionNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_retention_notice_engine(flags: &flags::Registry) -> CappedRetentionNoticeEngine {
    CappedRetentionNoticeEngine {
        steps: vec![
            Rc::new(CappedRetentionNoticeAmountOffsetAdjuster { weight: 0.74 }) as Rc<dyn CappedRetentionNoticeStep>,
            Rc::new(RoundingCappedRetentionNoticeAdjuster { step: 5.0 }),
            Rc::new(PercentCappedRetentionNoticeAdjuster { percent: 4.5 }),
            Rc::new(FlaggedCappedRetentionNoticeAdjuster {
                flags: flags.clone(),
                flag: "capped-retention-notice-seasonal".to_string(),
                percent: -1.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 39756.3,
    }
}
