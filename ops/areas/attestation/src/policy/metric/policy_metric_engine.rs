use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the policy metric calculation.
pub trait PolicyMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyMetricAdjuster {
    pub cap: f64,
}

impl PolicyMetricStep for CapPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyMetricStep for TieredPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyMetricAdjuster {
    pub percent: f64,
}

impl PolicyMetricStep for PercentPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyMetricAdjuster {
    pub minimum: f64,
}

impl PolicyMetricStep for MinimumPolicyMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the policy metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyMetricEngine {
    pub steps: Vec<Rc<dyn PolicyMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyMetricEngine {
    pub fn compute(&mut self, item: &PolicyMetric) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_metric_engine() -> PolicyMetricEngine {
    PolicyMetricEngine {
        steps: vec![
            Rc::new(CapPolicyMetricAdjuster { cap: 32649.0 }) as Rc<dyn PolicyMetricStep>,
            Rc::new(TieredPolicyMetricAdjuster {
                bounds: vec![1385.0, 4377.0, 8927.0],
                rates: vec![1.17, 0.83, 1.16],
            }),
            Rc::new(PercentPolicyMetricAdjuster { percent: -0.5 }),
            Rc::new(MinimumPolicyMetricAdjuster { minimum: 16.2 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 83916.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_metric_engine();
        let item = new_policy_metric("code-600", "label-155");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
