use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched segment quota calculation.
pub trait BatchedSegmentQuotaAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleBatchedSegmentQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSegmentQuotaAdjuster for TitleBatchedSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentQuotaCeilingOffsetAdjuster {
    pub weight: f64,
}

impl BatchedSegmentQuotaAdjuster for BatchedSegmentQuotaCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSegmentQuotaAdjuster {
    pub cap: f64,
}

impl BatchedSegmentQuotaAdjuster for CapBatchedSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedSegmentQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedSegmentQuotaAdjuster for TieredBatchedSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedSegmentQuotaAdjuster {
    pub minimum: f64,
}

impl BatchedSegmentQuotaAdjuster for MinimumBatchedSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedSegmentQuotaAdjuster {
    pub step: f64,
}

impl BatchedSegmentQuotaAdjuster for RoundingBatchedSegmentQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedSegmentQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the batched segment quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSegmentQuotaEngine {
    pub steps: Vec<Rc<dyn BatchedSegmentQuotaAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSegmentQuotaEngine {
    pub fn run(&mut self, item: &BatchedSegmentQuota) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSegmentQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_segment_quota_engine() -> BatchedSegmentQuotaEngine {
    BatchedSegmentQuotaEngine {
        steps: vec![
            Rc::new(TitleBatchedSegmentQuotaAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.21)]),
            }) as Rc<dyn BatchedSegmentQuotaAdjuster>,
            Rc::new(BatchedSegmentQuotaCeilingOffsetAdjuster { weight: 1.1 }),
            Rc::new(CapBatchedSegmentQuotaAdjuster { cap: 32521.0 }),
            Rc::new(TieredBatchedSegmentQuotaAdjuster {
                bounds: vec![2910.0, 5809.0],
                rates: vec![0.95, 1.15],
            }),
            Rc::new(MinimumBatchedSegmentQuotaAdjuster { minimum: 18.0 }),
            Rc::new(RoundingBatchedSegmentQuotaAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 81771.5,
    }
}
