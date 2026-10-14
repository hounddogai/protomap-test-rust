use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe batch calculation.
pub trait ProbeBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeBatchMarginOffsetAdjuster {
    pub weight: f64,
}

impl ProbeBatchModifier for ProbeBatchMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeBatch, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeBatchAdjuster {
    pub cap: f64,
}

impl ProbeBatchModifier for CapProbeBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumProbeBatchAdjuster {
    pub minimum: f64,
}

impl ProbeBatchModifier for MinimumProbeBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ProbeBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the probe batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeBatchEngine {
    pub steps: Vec<Rc<dyn ProbeBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeBatchEngine {
    pub fn price(&mut self, item: &ProbeBatch) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[ProbeBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_batch_engine() -> ProbeBatchEngine {
    ProbeBatchEngine {
        steps: vec![
            Rc::new(ProbeBatchMarginOffsetAdjuster { weight: 1.04 }) as Rc<dyn ProbeBatchModifier>,
            Rc::new(CapProbeBatchAdjuster { cap: 2427.0 }),
            Rc::new(MinimumProbeBatchAdjuster { minimum: 22.6 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 59137.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_probe_batch_engine();
        let item = new_probe_batch("code-238", "name-440");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
