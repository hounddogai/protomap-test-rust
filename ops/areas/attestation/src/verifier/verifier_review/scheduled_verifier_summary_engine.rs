use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled verifier summary calculation.
pub trait ScheduledVerifierSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledVerifierSummaryAdjuster {
    pub percent: f64,
}

impl ScheduledVerifierSummaryFactor for PercentScheduledVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledVerifierSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledVerifierSummaryFactor for TieredScheduledVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledVerifierSummaryAdjuster {
    pub step: f64,
}

impl ScheduledVerifierSummaryFactor for RoundingScheduledVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledVerifierSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledVerifierSummaryFactor for FlaggedScheduledVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledVerifierSummaryAdjuster {
    pub cap: f64,
}

impl ScheduledVerifierSummaryFactor for CapScheduledVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummaryLimitOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledVerifierSummaryFactor for ScheduledVerifierSummaryLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledVerifierSummary, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the scheduled verifier summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledVerifierSummaryEngine {
    pub steps: Vec<Rc<dyn ScheduledVerifierSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledVerifierSummaryEngine {
    pub fn evaluate(&mut self, item: &ScheduledVerifierSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledVerifierSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_scheduled_verifier_summary_engine(flags: &flags::Registry) -> ScheduledVerifierSummaryEngine {
    ScheduledVerifierSummaryEngine {
        steps: vec![
            Rc::new(PercentScheduledVerifierSummaryAdjuster { percent: -3.9 })
                as Rc<dyn ScheduledVerifierSummaryFactor>,
            Rc::new(TieredScheduledVerifierSummaryAdjuster {
                bounds: vec![1150.0, 3593.0, 6963.0, 8891.0],
                rates: vec![1.04, 1.19, 0.95, 1.14],
            }),
            Rc::new(RoundingScheduledVerifierSummaryAdjuster { step: 5.0 }),
            Rc::new(FlaggedScheduledVerifierSummaryAdjuster {
                flags: flags.clone(),
                flag: "scheduled-verifier-summary-batched".to_string(),
                percent: 1.0,
            }),
            Rc::new(CapScheduledVerifierSummaryAdjuster { cap: 7130.0 }),
            Rc::new(ScheduledVerifierSummaryLimitOffsetAdjuster { weight: 0.57 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 21493.9,
    }
}
