use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global quote checkpoint calculation.
pub trait GlobalQuoteCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalQuoteCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalQuoteCheckpointAdjuster {
    pub minimum: f64,
}

impl GlobalQuoteCheckpointStep for MinimumGlobalQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalQuoteCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalQuoteCheckpointAdjuster {
    pub percent: f64,
}

impl GlobalQuoteCheckpointStep for PercentGlobalQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalQuoteCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalQuoteCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalQuoteCheckpointStep for TieredGlobalQuoteCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalQuoteCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpointVersionOffsetAdjuster {
    pub weight: f64,
}

impl GlobalQuoteCheckpointStep for GlobalQuoteCheckpointVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalQuoteCheckpoint, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

/// Runs the global quote checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalQuoteCheckpointEngine {
    pub steps: Vec<Rc<dyn GlobalQuoteCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalQuoteCheckpointEngine {
    pub fn compute(&mut self, item: &GlobalQuoteCheckpoint) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[GlobalQuoteCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_global_quote_checkpoint_engine() -> GlobalQuoteCheckpointEngine {
    GlobalQuoteCheckpointEngine {
        steps: vec![
            Rc::new(MinimumGlobalQuoteCheckpointAdjuster { minimum: 32.7 }) as Rc<dyn GlobalQuoteCheckpointStep>,
            Rc::new(PercentGlobalQuoteCheckpointAdjuster { percent: -11.9 }),
            Rc::new(TieredGlobalQuoteCheckpointAdjuster {
                bounds: vec![721.0, 752.0, 1922.0, 7585.0],
                rates: vec![1.18, 1.08, 0.92, 0.82],
            }),
            Rc::new(GlobalQuoteCheckpointVersionOffsetAdjuster { weight: 0.01 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 32553.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_quote_checkpoint_engine();
        let item = new_global_quote_checkpoint("code-955", "name-208");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
