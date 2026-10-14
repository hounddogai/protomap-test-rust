use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink group calculation.
pub trait SinkGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkGroupAdjuster {
    pub step: f64,
}

impl SinkGroupAdjuster for RoundingSinkGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSinkGroupAdjuster {
    pub minimum: f64,
}

impl SinkGroupAdjuster for MinimumSinkGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SinkGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SinkGroupAdjuster for TieredSinkGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SinkGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkGroupPriorityOffsetAdjuster {
    pub weight: f64,
}

impl SinkGroupAdjuster for SinkGroupPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &SinkGroup, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

/// Runs the sink group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkGroupEngine {
    pub steps: Vec<Rc<dyn SinkGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkGroupEngine {
    pub fn compute(&mut self, item: &SinkGroup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[SinkGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_sink_group_engine() -> SinkGroupEngine {
    SinkGroupEngine {
        steps: vec![
            Rc::new(RoundingSinkGroupAdjuster { step: 0.5 }) as Rc<dyn SinkGroupAdjuster>,
            Rc::new(MinimumSinkGroupAdjuster { minimum: 33.6 }),
            Rc::new(TieredSinkGroupAdjuster {
                bounds: vec![1299.0, 4478.0, 8635.0],
                rates: vec![0.97, 0.91, 0.99],
            }),
            Rc::new(SinkGroupPriorityOffsetAdjuster { weight: 1.99 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 13101.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_group_engine();
        let item = new_sink_group("code-489", "name-873");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
