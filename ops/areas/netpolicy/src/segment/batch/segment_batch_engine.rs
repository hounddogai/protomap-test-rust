use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment batch calculation.
pub trait SegmentBatchAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentBatchAdjuster {
    pub percent: f64,
}

impl SegmentBatchAdjuster for PercentSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategorySegmentBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentBatchAdjuster for CategorySegmentBatchAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBatchPortOffsetAdjuster {
    pub weight: f64,
}

impl SegmentBatchAdjuster for SegmentBatchPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentBatchAdjuster {
    pub step: f64,
}

impl SegmentBatchAdjuster for RoundingSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentBatchAdjuster {
    pub cap: f64,
}

impl SegmentBatchAdjuster for CapSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentBatchAdjuster for TieredSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the segment batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentBatchEngine {
    pub steps: Vec<Rc<dyn SegmentBatchAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentBatchEngine {
    pub fn run(&mut self, item: &SegmentBatch) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SegmentBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_segment_batch_engine() -> SegmentBatchEngine {
    SegmentBatchEngine {
        steps: vec![
            Rc::new(PercentSegmentBatchAdjuster { percent: 23.4 }) as Rc<dyn SegmentBatchAdjuster>,
            Rc::new(CategorySegmentBatchAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.02), ("central".to_string(), 0.76)]),
            }),
            Rc::new(SegmentBatchPortOffsetAdjuster { weight: 1.86 }),
            Rc::new(RoundingSegmentBatchAdjuster { step: 0.25 }),
            Rc::new(CapSegmentBatchAdjuster { cap: 577.0 }),
            Rc::new(TieredSegmentBatchAdjuster {
                bounds: vec![2387.0, 3502.0, 4105.0, 7520.0],
                rates: vec![0.85, 0.81, 1.2, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 78349.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_batch_engine();
        let item = new_segment_batch("code-297", "name-350");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
