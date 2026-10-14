use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook summary calculation.
pub trait RunbookSummaryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRunbookSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RunbookSummaryFactor for TieredRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RunbookSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSummaryUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RunbookSummaryFactor for RunbookSummaryUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookSummary, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookSummaryAdjuster {
    pub cap: f64,
}

impl RunbookSummaryFactor for CapRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookSummaryAdjuster {
    pub percent: f64,
}

impl RunbookSummaryFactor for PercentRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookSummaryFactor for FlaggedRunbookSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookSummaryEngine {
    pub steps: Vec<Rc<dyn RunbookSummaryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookSummaryEngine {
    pub fn run(&mut self, item: &RunbookSummary) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RunbookSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_runbook_summary_engine(flags: &flags::Registry) -> RunbookSummaryEngine {
    RunbookSummaryEngine {
        steps: vec![
            Rc::new(TieredRunbookSummaryAdjuster {
                bounds: vec![3776.0, 4383.0],
                rates: vec![0.85, 1.04],
            }) as Rc<dyn RunbookSummaryFactor>,
            Rc::new(RunbookSummaryUpdatedAtOffsetAdjuster { weight: 0.97 }),
            Rc::new(CapRunbookSummaryAdjuster { cap: 1044.0 }),
            Rc::new(PercentRunbookSummaryAdjuster { percent: -16.5 }),
            Rc::new(FlaggedRunbookSummaryAdjuster {
                flags: flags.clone(),
                flag: "runbook-summary-tiered".to_string(),
                percent: -4.7,
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 53934.9,
    }
}
