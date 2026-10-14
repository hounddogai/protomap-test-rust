use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the suspension baseline calculation.
pub trait SuspensionBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SuspensionBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSuspensionBaselineAdjuster {
    pub step: f64,
}

impl SuspensionBaselineAdjuster for RoundingSuspensionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SuspensionBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSuspensionBaselineAdjuster {
    pub minimum: f64,
}

impl SuspensionBaselineAdjuster for MinimumSuspensionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SuspensionBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSuspensionBaselineAdjuster {
    pub percent: f64,
}

impl SuspensionBaselineAdjuster for PercentSuspensionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SuspensionBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionBaselineUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl SuspensionBaselineAdjuster for SuspensionBaselineUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &SuspensionBaseline, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the suspension baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SuspensionBaselineEngine {
    pub steps: Vec<Rc<dyn SuspensionBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SuspensionBaselineEngine {
    pub fn price(&mut self, item: &SuspensionBaseline) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SuspensionBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_suspension_baseline_engine() -> SuspensionBaselineEngine {
    SuspensionBaselineEngine {
        steps: vec![
            Rc::new(RoundingSuspensionBaselineAdjuster { step: 1.0 }) as Rc<dyn SuspensionBaselineAdjuster>,
            Rc::new(MinimumSuspensionBaselineAdjuster { minimum: 24.2 }),
            Rc::new(PercentSuspensionBaselineAdjuster { percent: -2.1 }),
            Rc::new(SuspensionBaselineUpdatedAtOffsetAdjuster { weight: 0.49 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 30727.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_suspension_baseline_engine();
        let item = new_suspension_baseline("code-535", "name-750");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
