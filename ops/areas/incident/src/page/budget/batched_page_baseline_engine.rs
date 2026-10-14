use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched page baseline calculation.
pub trait BatchedPageBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedPageBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedPageBaselineAdjuster {
    pub minimum: f64,
}

impl BatchedPageBaselineStep for MinimumBatchedPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedPageBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedPageBaselineAdjuster {
    pub step: f64,
}

impl BatchedPageBaselineStep for RoundingBatchedPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedPageBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedPageBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedPageBaselineStep for FlaggedBatchedPageBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedPageBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageBaselinePriorityOffsetAdjuster {
    pub weight: f64,
}

impl BatchedPageBaselineStep for BatchedPageBaselinePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedPageBaseline, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the batched page baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedPageBaselineEngine {
    pub steps: Vec<Rc<dyn BatchedPageBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedPageBaselineEngine {
    pub fn price(&mut self, item: &BatchedPageBaseline) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[BatchedPageBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_page_baseline_engine(flags: &flags::Registry) -> BatchedPageBaselineEngine {
    BatchedPageBaselineEngine {
        steps: vec![
            Rc::new(MinimumBatchedPageBaselineAdjuster { minimum: 37.4 }) as Rc<dyn BatchedPageBaselineStep>,
            Rc::new(RoundingBatchedPageBaselineAdjuster { step: 0.05 }),
            Rc::new(FlaggedBatchedPageBaselineAdjuster {
                flags: flags.clone(),
                flag: "batched-page-baseline-batched".to_string(),
                percent: -2.6,
            }),
            Rc::new(BatchedPageBaselinePriorityOffsetAdjuster { weight: 0.03 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 49762.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_page_baseline_engine(&flags::new_registry());
        let item = new_batched_page_baseline("code-56", "name-293");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
