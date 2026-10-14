use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the query request calculation.
pub trait QueryRequestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QueryRequest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQueryRequestAdjuster {
    pub cap: f64,
}

impl QueryRequestAdjuster for CapQueryRequestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QueryRequest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQueryRequestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QueryRequestAdjuster for TieredQueryRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QueryRequest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQueryRequestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QueryRequestAdjuster for FlaggedQueryRequestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QueryRequest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequestCapacityOffsetAdjuster {
    pub weight: f64,
}

impl QueryRequestAdjuster for QueryRequestCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &QueryRequest, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

/// Runs the query request steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QueryRequestEngine {
    pub steps: Vec<Rc<dyn QueryRequestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QueryRequestEngine {
    pub fn compute(&mut self, item: &QueryRequest) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QueryRequestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QueryRequest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_query_request_engine(flags: &flags::Registry) -> QueryRequestEngine {
    QueryRequestEngine {
        steps: vec![
            Rc::new(CapQueryRequestAdjuster { cap: 44772.0 }) as Rc<dyn QueryRequestAdjuster>,
            Rc::new(TieredQueryRequestAdjuster {
                bounds: vec![6964.0, 8480.0, 9256.0],
                rates: vec![1.11, 1.05, 1.15],
            }),
            Rc::new(FlaggedQueryRequestAdjuster {
                flags: flags.clone(),
                flag: "query-request-scheduled".to_string(),
                percent: 2.6,
            }),
            Rc::new(QueryRequestCapacityOffsetAdjuster { weight: 1.58 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 43952.9,
    }
}
