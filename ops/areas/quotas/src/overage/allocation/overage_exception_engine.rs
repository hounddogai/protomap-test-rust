use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage exception calculation.
pub trait OverageExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl OverageExceptionAdjuster for TieredOverageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &OverageException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageExceptionAdjuster for FlaggedOverageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapOverageExceptionAdjuster {
    pub cap: f64,
}

impl OverageExceptionAdjuster for CapOverageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &OverageException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageExceptionAdjuster {
    pub percent: f64,
}

impl OverageExceptionAdjuster for PercentOverageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageExceptionRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl OverageExceptionAdjuster for OverageExceptionRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &OverageException, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageExceptionAdjuster {
    pub step: f64,
}

impl OverageExceptionAdjuster for RoundingOverageExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageExceptionEngine {
    pub steps: Vec<Rc<dyn OverageExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageExceptionEngine {
    pub fn compute(&mut self, item: &OverageException) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageExceptionAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[OverageException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_overage_exception_engine(flags: &flags::Registry) -> OverageExceptionEngine {
    OverageExceptionEngine {
        steps: vec![
            Rc::new(TieredOverageExceptionAdjuster {
                bounds: vec![2208.0, 7515.0],
                rates: vec![1.09, 0.96],
            }) as Rc<dyn OverageExceptionAdjuster>,
            Rc::new(FlaggedOverageExceptionAdjuster {
                flags: flags.clone(),
                flag: "overage-exception-manual".to_string(),
                percent: 4.8,
            }),
            Rc::new(CapOverageExceptionAdjuster { cap: 6495.0 }),
            Rc::new(PercentOverageExceptionAdjuster { percent: -3.3 }),
            Rc::new(OverageExceptionRotationDaysOffsetAdjuster { weight: 1.28 }),
            Rc::new(RoundingOverageExceptionAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 52641.2,
    }
}
