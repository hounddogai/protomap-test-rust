use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched forecast cycle calculation.
pub trait BatchedForecastCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedForecastCycle, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedForecastCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedForecastCycleStep for FlaggedBatchedForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedForecastCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedForecastCycleAdjuster {
    pub minimum: f64,
}

impl BatchedForecastCycleStep for MinimumBatchedForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedForecastCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedForecastCycleAdjuster {
    pub step: f64,
}

impl BatchedForecastCycleStep for RoundingBatchedForecastCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedForecastCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched forecast cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedForecastCycleEngine {
    pub steps: Vec<Rc<dyn BatchedForecastCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedForecastCycleEngine {
    pub fn compute(&mut self, item: &BatchedForecastCycle) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedForecastCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_forecast_cycle_engine(flags: &flags::Registry) -> BatchedForecastCycleEngine {
    BatchedForecastCycleEngine {
        steps: vec![
            Rc::new(FlaggedBatchedForecastCycleAdjuster {
                flags: flags.clone(),
                flag: "batched-forecast-cycle-scheduled".to_string(),
                percent: -4.3,
            }) as Rc<dyn BatchedForecastCycleStep>,
            Rc::new(MinimumBatchedForecastCycleAdjuster { minimum: 33.9 }),
            Rc::new(RoundingBatchedForecastCycleAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 60542.0,
    }
}
