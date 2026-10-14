use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global federation notice calculation.
pub trait GlobalFederationNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalFederationNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalFederationNoticeAdjuster {
    pub step: f64,
}

impl GlobalFederationNoticeStep for RoundingGlobalFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalFederationNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalFederationNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalFederationNoticeStep for TieredGlobalFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalFederationNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalFederationNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalFederationNoticeStep for FlaggedGlobalFederationNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalFederationNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global federation notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalFederationNoticeEngine {
    pub steps: Vec<Rc<dyn GlobalFederationNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalFederationNoticeEngine {
    pub fn run(&mut self, item: &GlobalFederationNotice) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalFederationNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_federation_notice_engine(flags: &flags::Registry) -> GlobalFederationNoticeEngine {
    GlobalFederationNoticeEngine {
        steps: vec![
            Rc::new(RoundingGlobalFederationNoticeAdjuster { step: 5.0 }) as Rc<dyn GlobalFederationNoticeStep>,
            Rc::new(TieredGlobalFederationNoticeAdjuster {
                bounds: vec![2557.0, 6156.0, 8415.0],
                rates: vec![0.81, 0.81, 0.95],
            }),
            Rc::new(FlaggedGlobalFederationNoticeAdjuster {
                flags: flags.clone(),
                flag: "global-federation-notice-capped".to_string(),
                percent: 6.3,
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 3961.2,
    }
}
