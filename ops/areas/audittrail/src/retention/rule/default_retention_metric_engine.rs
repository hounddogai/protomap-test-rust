use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the default retention metric calculation.
pub trait DefaultRetentionMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultRetentionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultRetentionMetricAdjuster {
    pub step: f64,
}

impl DefaultRetentionMetricFactor for RoundingDefaultRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultRetentionMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultRetentionMetricAdjuster {
    pub minimum: f64,
}

impl DefaultRetentionMetricFactor for MinimumDefaultRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultRetentionMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultRetentionMetricAdjuster {
    pub cap: f64,
}

impl DefaultRetentionMetricFactor for CapDefaultRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultRetentionMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the default retention metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultRetentionMetricEngine {
    pub steps: Vec<Rc<dyn DefaultRetentionMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultRetentionMetricEngine {
    pub fn evaluate(&mut self, item: &DefaultRetentionMetric) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultRetentionMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultRetentionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_retention_metric_engine() -> DefaultRetentionMetricEngine {
    DefaultRetentionMetricEngine {
        steps: vec![
            Rc::new(RoundingDefaultRetentionMetricAdjuster { step: 1.0 }) as Rc<dyn DefaultRetentionMetricFactor>,
            Rc::new(MinimumDefaultRetentionMetricAdjuster { minimum: 32.5 }),
            Rc::new(CapDefaultRetentionMetricAdjuster { cap: 18233.0 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 31498.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_retention_metric_engine();
        let item = new_default_retention_metric("code-94", "name-356");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
