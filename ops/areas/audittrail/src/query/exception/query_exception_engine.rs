use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query exception calculation.
pub trait QueryExceptionStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryExceptionAdjuster {
    pub cap: f64,
}

impl QueryExceptionStep for CapQueryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQueryExceptionAdjuster {
    pub minimum: f64,
}

impl QueryExceptionStep for MinimumQueryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QueryException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQueryExceptionAdjuster {
    pub step: f64,
}

impl QueryExceptionStep for RoundingQueryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QueryException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryExceptionAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryExceptionStep for TieredQueryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryException, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryExceptionStep for FlaggedQueryExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryExceptionEngine {
    pub steps: Vec<Rc<dyn QueryExceptionStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryExceptionEngine {
    pub fn run(&mut self, item: &QueryException) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[QueryException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_query_exception_engine(flags: &flags::Registry) -> QueryExceptionEngine {
    QueryExceptionEngine {
        steps: vec![
            Rc::new(CapQueryExceptionAdjuster { cap: 963.0 }) as Rc<dyn QueryExceptionStep>,
            Rc::new(MinimumQueryExceptionAdjuster { minimum: 24.0 }),
            Rc::new(RoundingQueryExceptionAdjuster { step: 0.1 }),
            Rc::new(TieredQueryExceptionAdjuster {
                bounds: vec![9548.0, 9978.0],
                rates: vec![1.05, 0.8],
            }),
            Rc::new(FlaggedQueryExceptionAdjuster {
                flags: flags.clone(),
                flag: "query-exception-global".to_string(),
                percent: 6.4,
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 88969.1,
    }
}
