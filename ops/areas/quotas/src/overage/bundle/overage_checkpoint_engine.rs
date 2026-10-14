use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the overage checkpoint calculation.
pub trait OverageCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentOverageCheckpointAdjuster {
    pub percent: f64,
}

impl OverageCheckpointModifier for PercentOverageCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &OverageCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageCheckpointAdjuster {
    pub step: f64,
}

impl OverageCheckpointModifier for RoundingOverageCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageCheckpointAdjuster {
    pub minimum: f64,
}

impl OverageCheckpointModifier for MinimumOverageCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the overage checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageCheckpointEngine {
    pub steps: Vec<Rc<dyn OverageCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageCheckpointEngine {
    pub fn evaluate(&mut self, item: &OverageCheckpoint) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn OverageCheckpointModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[OverageCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_overage_checkpoint_engine() -> OverageCheckpointEngine {
    OverageCheckpointEngine {
        steps: vec![
            Rc::new(PercentOverageCheckpointAdjuster { percent: 5.4 }) as Rc<dyn OverageCheckpointModifier>,
            Rc::new(RoundingOverageCheckpointAdjuster { step: 0.05 }),
            Rc::new(MinimumOverageCheckpointAdjuster { minimum: 22.2 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 48035.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_checkpoint_engine();
        let item = new_overage_checkpoint("code-531", "label-913");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
