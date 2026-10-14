use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the query segment calculation.
pub trait QuerySegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuerySegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedQuerySegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuerySegmentFactor for FlaggedQuerySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuerySegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuerySegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuerySegmentFactor for TieredQuerySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuerySegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelQuerySegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl QuerySegmentFactor for LabelQuerySegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &QuerySegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuerySegmentAdjuster {
    pub step: f64,
}

impl QuerySegmentFactor for RoundingQuerySegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuerySegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentQuerySegmentAdjuster {
    pub percent: f64,
}

impl QuerySegmentFactor for PercentQuerySegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &QuerySegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the query segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuerySegmentEngine {
    pub steps: Vec<Rc<dyn QuerySegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuerySegmentEngine {
    pub fn evaluate(&mut self, item: &QuerySegment) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn QuerySegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[QuerySegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_query_segment_engine(flags: &flags::Registry) -> QuerySegmentEngine {
    QuerySegmentEngine {
        steps: vec![
            Rc::new(FlaggedQuerySegmentAdjuster {
                flags: flags.clone(),
                flag: "query-segment-tiered".to_string(),
                percent: 6.7,
            }) as Rc<dyn QuerySegmentFactor>,
            Rc::new(TieredQuerySegmentAdjuster {
                bounds: vec![1700.0, 2042.0],
                rates: vec![1.0, 1.05],
            }),
            Rc::new(LabelQuerySegmentAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.13),
                    ("online".to_string(), 1.04),
                    ("east".to_string(), 0.85),
                ]),
            }),
            Rc::new(RoundingQuerySegmentAdjuster { step: 0.1 }),
            Rc::new(PercentQuerySegmentAdjuster { percent: -15.7 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 46520.4,
    }
}
