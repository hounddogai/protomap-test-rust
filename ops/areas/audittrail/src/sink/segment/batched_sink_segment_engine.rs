use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched sink segment calculation.
pub trait BatchedSinkSegmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedSinkSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedSinkSegmentAdjuster {
    pub percent: f64,
}

impl BatchedSinkSegmentModifier for PercentBatchedSinkSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedSinkSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedSinkSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedSinkSegmentModifier for TieredBatchedSinkSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedSinkSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedSinkSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedSinkSegmentModifier for LabelBatchedSinkSegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedSinkSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedSinkSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedSinkSegmentModifier for FlaggedBatchedSinkSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedSinkSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedSinkSegmentAdjuster {
    pub minimum: f64,
}

impl BatchedSinkSegmentModifier for MinimumBatchedSinkSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedSinkSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the batched sink segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedSinkSegmentEngine {
    pub steps: Vec<Rc<dyn BatchedSinkSegmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedSinkSegmentEngine {
    pub fn price(&mut self, item: &BatchedSinkSegment) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedSinkSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_sink_segment_engine(flags: &flags::Registry) -> BatchedSinkSegmentEngine {
    BatchedSinkSegmentEngine {
        steps: vec![
            Rc::new(PercentBatchedSinkSegmentAdjuster { percent: -20.4 }) as Rc<dyn BatchedSinkSegmentModifier>,
            Rc::new(TieredBatchedSinkSegmentAdjuster {
                bounds: vec![2871.0, 8661.0, 9688.0, 9956.0],
                rates: vec![0.93, 0.83, 1.18, 1.06],
            }),
            Rc::new(LabelBatchedSinkSegmentAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.16)]),
            }),
            Rc::new(FlaggedBatchedSinkSegmentAdjuster {
                flags: flags.clone(),
                flag: "batched-sink-segment-batched".to_string(),
                percent: -3.8,
            }),
            Rc::new(MinimumBatchedSinkSegmentAdjuster { minimum: 5.2 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 87981.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_sink_segment_engine(&flags::new_registry());
        let item = new_batched_sink_segment("code-429", "label-879");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
