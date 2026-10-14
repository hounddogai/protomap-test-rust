use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the measurement budget calculation.
pub trait MeasurementBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementBudgetAdjuster {
    pub percent: f64,
}

impl MeasurementBudgetStep for PercentMeasurementBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementBudgetAdjuster {
    pub cap: f64,
}

impl MeasurementBudgetStep for CapMeasurementBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBudgetAmountOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementBudgetStep for MeasurementBudgetAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementBudget, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementBudgetStep for FlaggedMeasurementBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementBudgetStep for TieredMeasurementBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the measurement budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementBudgetEngine {
    pub steps: Vec<Rc<dyn MeasurementBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementBudgetEngine {
    pub fn compute(&mut self, item: &MeasurementBudget) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_budget_engine(flags: &flags::Registry) -> MeasurementBudgetEngine {
    MeasurementBudgetEngine {
        steps: vec![
            Rc::new(PercentMeasurementBudgetAdjuster { percent: 23.3 }) as Rc<dyn MeasurementBudgetStep>,
            Rc::new(CapMeasurementBudgetAdjuster { cap: 14701.0 }),
            Rc::new(MeasurementBudgetAmountOffsetAdjuster { weight: 1.11 }),
            Rc::new(FlaggedMeasurementBudgetAdjuster {
                flags: flags.clone(),
                flag: "measurement-budget-regional".to_string(),
                percent: 1.5,
            }),
            Rc::new(TieredMeasurementBudgetAdjuster {
                bounds: vec![4731.0, 5643.0],
                rates: vec![0.8, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 43646.4,
    }
}
