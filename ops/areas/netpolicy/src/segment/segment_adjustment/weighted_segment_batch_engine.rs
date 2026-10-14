use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted segment batch calculation.
pub trait WeightedSegmentBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedSegmentBatchAdjuster {
    pub cap: f64,
}

impl WeightedSegmentBatchFactor for CapWeightedSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedSegmentBatchAdjuster {
    pub step: f64,
}

impl WeightedSegmentBatchFactor for RoundingWeightedSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedSegmentBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedSegmentBatchFactor for LabelWeightedSegmentBatchAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedSegmentBatchAdjuster {
    pub minimum: f64,
}

impl WeightedSegmentBatchFactor for MinimumWeightedSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedSegmentBatchAdjuster {
    pub percent: f64,
}

impl WeightedSegmentBatchFactor for PercentWeightedSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedSegmentBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedSegmentBatchFactor for FlaggedWeightedSegmentBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedSegmentBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the weighted segment batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedSegmentBatchEngine {
    pub steps: Vec<Rc<dyn WeightedSegmentBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedSegmentBatchEngine {
    pub fn price(&mut self, item: &WeightedSegmentBatch) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[WeightedSegmentBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_segment_batch_engine(flags: &flags::Registry) -> WeightedSegmentBatchEngine {
    WeightedSegmentBatchEngine {
        steps: vec![
            Rc::new(CapWeightedSegmentBatchAdjuster { cap: 5862.0 }) as Rc<dyn WeightedSegmentBatchFactor>,
            Rc::new(RoundingWeightedSegmentBatchAdjuster { step: 0.5 }),
            Rc::new(LabelWeightedSegmentBatchAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.09), ("online".to_string(), 0.93)]),
            }),
            Rc::new(MinimumWeightedSegmentBatchAdjuster { minimum: 33.4 }),
            Rc::new(PercentWeightedSegmentBatchAdjuster { percent: -12.5 }),
            Rc::new(FlaggedWeightedSegmentBatchAdjuster {
                flags: flags.clone(),
                flag: "weighted-segment-batch-pending".to_string(),
                percent: 2.6,
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 11572.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_segment_batch_engine(&flags::new_registry());
        let item = new_weighted_segment_batch("code-638", "label-861");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
