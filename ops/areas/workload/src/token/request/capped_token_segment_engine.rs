use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped token segment calculation.
pub trait CappedTokenSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedTokenSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCappedTokenSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedTokenSegmentFactor for SegmentCappedTokenSegmentAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CappedTokenSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedTokenSegmentAdjuster {
    pub cap: f64,
}

impl CappedTokenSegmentFactor for CapCappedTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedTokenSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedTokenSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedTokenSegmentFactor for TieredCappedTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedTokenSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedTokenSegmentAdjuster {
    pub minimum: f64,
}

impl CappedTokenSegmentFactor for MinimumCappedTokenSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedTokenSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the capped token segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedTokenSegmentEngine {
    pub steps: Vec<Rc<dyn CappedTokenSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedTokenSegmentEngine {
    pub fn compute(&mut self, item: &CappedTokenSegment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CappedTokenSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_token_segment_engine() -> CappedTokenSegmentEngine {
    CappedTokenSegmentEngine {
        steps: vec![
            Rc::new(SegmentCappedTokenSegmentAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.34),
                    ("south".to_string(), 0.98),
                    ("central".to_string(), 1.11),
                ]),
            }) as Rc<dyn CappedTokenSegmentFactor>,
            Rc::new(CapCappedTokenSegmentAdjuster { cap: 14150.0 }),
            Rc::new(TieredCappedTokenSegmentAdjuster {
                bounds: vec![5235.0, 7269.0],
                rates: vec![1.13, 1.17],
            }),
            Rc::new(MinimumCappedTokenSegmentAdjuster { minimum: 6.3 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 27582.5,
    }
}
