use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the failover summary calculation.
pub trait FailoverSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverSummary, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFailoverSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FailoverSummaryStep for FlaggedFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FailoverSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverSummaryStep for TieredFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverSummaryAdjuster {
    pub step: f64,
}

impl FailoverSummaryStep for RoundingFailoverSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the failover summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverSummaryEngine {
    pub steps: Vec<Rc<dyn FailoverSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverSummaryEngine {
    pub fn run(&mut self, item: &FailoverSummary) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_failover_summary_engine(flags: &flags::Registry) -> FailoverSummaryEngine {
    FailoverSummaryEngine {
        steps: vec![
            Rc::new(FlaggedFailoverSummaryAdjuster {
                flags: flags.clone(),
                flag: "failover-summary-global".to_string(),
                percent: 0.3,
            }) as Rc<dyn FailoverSummaryStep>,
            Rc::new(TieredFailoverSummaryAdjuster {
                bounds: vec![338.0, 2581.0, 2650.0, 5000.0],
                rates: vec![1.1, 0.94, 0.97, 0.92],
            }),
            Rc::new(RoundingFailoverSummaryAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 84268.9,
    }
}
