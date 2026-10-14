use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched region pin window calculation.
pub trait BatchedRegionPinWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinWindowRateOffsetAdjuster {
    pub weight: f64,
}

impl BatchedRegionPinWindowAdjuster for BatchedRegionPinWindowRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedRegionPinWindowAdjuster {
    pub step: f64,
}

impl BatchedRegionPinWindowAdjuster for RoundingBatchedRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedRegionPinWindowAdjuster {
    pub minimum: f64,
}

impl BatchedRegionPinWindowAdjuster for MinimumBatchedRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedRegionPinWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedRegionPinWindowAdjuster for TieredBatchedRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedRegionPinWindowAdjuster {
    pub percent: f64,
}

impl BatchedRegionPinWindowAdjuster for PercentBatchedRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBatchedRegionPinWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedRegionPinWindowAdjuster for LabelBatchedRegionPinWindowAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BatchedRegionPinWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the batched region pin window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedRegionPinWindowEngine {
    pub steps: Vec<Rc<dyn BatchedRegionPinWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedRegionPinWindowEngine {
    pub fn price(&mut self, item: &BatchedRegionPinWindow) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[BatchedRegionPinWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_batched_region_pin_window_engine() -> BatchedRegionPinWindowEngine {
    BatchedRegionPinWindowEngine {
        steps: vec![
            Rc::new(BatchedRegionPinWindowRateOffsetAdjuster { weight: 1.7 }) as Rc<dyn BatchedRegionPinWindowAdjuster>,
            Rc::new(RoundingBatchedRegionPinWindowAdjuster { step: 5.0 }),
            Rc::new(MinimumBatchedRegionPinWindowAdjuster { minimum: 47.7 }),
            Rc::new(TieredBatchedRegionPinWindowAdjuster {
                bounds: vec![2119.0, 3914.0, 6080.0, 7604.0],
                rates: vec![1.05, 0.81, 0.94, 0.85],
            }),
            Rc::new(PercentBatchedRegionPinWindowAdjuster { percent: -15.0 }),
            Rc::new(LabelBatchedRegionPinWindowAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.87),
                    ("online".to_string(), 1.2),
                    ("east".to_string(), 1.27),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 28681.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_region_pin_window_engine();
        let item = new_batched_region_pin_window("code-76", "label-417");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
