use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending token metric calculation.
pub trait PendingTokenMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTokenMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingTokenMetricAdjuster {
    pub step: f64,
}

impl PendingTokenMetricStep for RoundingPendingTokenMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingTokenMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTokenMetricAdjuster {
    pub cap: f64,
}

impl PendingTokenMetricStep for CapPendingTokenMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTokenMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingTokenMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingTokenMetricStep for NamePendingTokenMetricAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingTokenMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending token metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTokenMetricEngine {
    pub steps: Vec<Rc<dyn PendingTokenMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTokenMetricEngine {
    pub fn compute(&mut self, item: &PendingTokenMetric) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingTokenMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingTokenMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_token_metric_engine() -> PendingTokenMetricEngine {
    PendingTokenMetricEngine {
        steps: vec![
            Rc::new(RoundingPendingTokenMetricAdjuster { step: 0.5 }) as Rc<dyn PendingTokenMetricStep>,
            Rc::new(CapPendingTokenMetricAdjuster { cap: 19685.0 }),
            Rc::new(NamePendingTokenMetricAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.87)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 9535.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_token_metric_engine();
        let item = new_pending_token_metric("code-578", "name-299");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
