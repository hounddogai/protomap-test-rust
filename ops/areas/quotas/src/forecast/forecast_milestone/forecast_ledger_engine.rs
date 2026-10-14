use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the forecast ledger calculation.
pub trait ForecastLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastLedgerScoreOffsetAdjuster {
    pub weight: f64,
}

impl ForecastLedgerStep for ForecastLedgerScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelForecastLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ForecastLedgerStep for LabelForecastLedgerAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingForecastLedgerAdjuster {
    pub step: f64,
}

impl ForecastLedgerStep for RoundingForecastLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastLedgerAdjuster {
    pub cap: f64,
}

impl ForecastLedgerStep for CapForecastLedgerAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastLedgerAdjuster {
    pub minimum: f64,
}

impl ForecastLedgerStep for MinimumForecastLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedForecastLedgerAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ForecastLedgerStep for FlaggedForecastLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ForecastLedger, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the forecast ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastLedgerEngine {
    pub steps: Vec<Rc<dyn ForecastLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastLedgerEngine {
    pub fn run(&mut self, item: &ForecastLedger) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_forecast_ledger_engine(flags: &flags::Registry) -> ForecastLedgerEngine {
    ForecastLedgerEngine {
        steps: vec![
            Rc::new(ForecastLedgerScoreOffsetAdjuster { weight: 0.64 }) as Rc<dyn ForecastLedgerStep>,
            Rc::new(LabelForecastLedgerAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.18)]),
            }),
            Rc::new(RoundingForecastLedgerAdjuster { step: 1.0 }),
            Rc::new(CapForecastLedgerAdjuster { cap: 29515.0 }),
            Rc::new(MinimumForecastLedgerAdjuster { minimum: 29.2 }),
            Rc::new(FlaggedForecastLedgerAdjuster {
                flags: flags.clone(),
                flag: "forecast-ledger-legacy".to_string(),
                percent: -5.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 38939.0,
    }
}
