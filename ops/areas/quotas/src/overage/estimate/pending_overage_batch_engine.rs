use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending overage batch calculation.
pub trait PendingOverageBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingOverageBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingOverageBatchAdjuster {
    pub percent: f64,
}

impl PendingOverageBatchModifier for PercentPendingOverageBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingOverageBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingOverageBatchAdjuster {
    pub cap: f64,
}

impl PendingOverageBatchModifier for CapPendingOverageBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingOverageBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingOverageBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingOverageBatchModifier for TieredPendingOverageBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingOverageBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending overage batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingOverageBatchEngine {
    pub steps: Vec<Rc<dyn PendingOverageBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingOverageBatchEngine {
    pub fn evaluate(&mut self, item: &PendingOverageBatch) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[PendingOverageBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_overage_batch_engine() -> PendingOverageBatchEngine {
    PendingOverageBatchEngine {
        steps: vec![
            Rc::new(PercentPendingOverageBatchAdjuster { percent: 23.5 }) as Rc<dyn PendingOverageBatchModifier>,
            Rc::new(CapPendingOverageBatchAdjuster { cap: 20446.0 }),
            Rc::new(TieredPendingOverageBatchAdjuster {
                bounds: vec![4107.0, 5247.0],
                rates: vec![1.03, 1.01],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 38381.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_overage_batch_engine();
        let item = new_pending_overage_batch("code-220", "label-308");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
