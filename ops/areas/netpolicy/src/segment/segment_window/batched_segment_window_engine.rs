use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched segment window calculation.
pub trait BatchedSegmentWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSegmentWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentWindowTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl BatchedSegmentWindowAdjuster for BatchedSegmentWindowTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedSegmentWindow, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedSegmentWindowAdjuster {
    pub cap: f64,
}

impl BatchedSegmentWindowAdjuster for CapBatchedSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedSegmentWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedSegmentWindowAdjuster {
    pub minimum: f64,
}

impl BatchedSegmentWindowAdjuster for MinimumBatchedSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedSegmentWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedSegmentWindowAdjuster {
    pub step: f64,
}

impl BatchedSegmentWindowAdjuster for RoundingBatchedSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedSegmentWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedSegmentWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedSegmentWindowAdjuster for FlaggedBatchedSegmentWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedSegmentWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched segment window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSegmentWindowEngine {
    pub steps: Vec<Rc<dyn BatchedSegmentWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSegmentWindowEngine {
    pub fn price(&mut self, item: &BatchedSegmentWindow) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BatchedSegmentWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_segment_window_engine(flags: &flags::Registry) -> BatchedSegmentWindowEngine {
    BatchedSegmentWindowEngine {
        steps: vec![
            Rc::new(BatchedSegmentWindowTtlSecondsOffsetAdjuster { weight: 1.61 })
                as Rc<dyn BatchedSegmentWindowAdjuster>,
            Rc::new(CapBatchedSegmentWindowAdjuster { cap: 49918.0 }),
            Rc::new(MinimumBatchedSegmentWindowAdjuster { minimum: 3.0 }),
            Rc::new(RoundingBatchedSegmentWindowAdjuster { step: 0.1 }),
            Rc::new(FlaggedBatchedSegmentWindowAdjuster {
                flags: flags.clone(),
                flag: "batched-segment-window-batched".to_string(),
                percent: -0.5,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 3966.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_segment_window_engine(&flags::new_registry());
        let item = new_batched_segment_window("code-354", "name-162");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
