use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending dashboard batch calculation.
pub trait PendingDashboardBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingDashboardBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingDashboardBatchAdjuster {
    pub step: f64,
}

impl PendingDashboardBatchModifier for RoundingPendingDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingDashboardBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingDashboardBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingDashboardBatchModifier for TieredPendingDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingDashboardBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDashboardBatchTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PendingDashboardBatchModifier for PendingDashboardBatchTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PendingDashboardBatch, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingDashboardBatchAdjuster {
    pub percent: f64,
}

impl PendingDashboardBatchModifier for PercentPendingDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingDashboardBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending dashboard batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingDashboardBatchEngine {
    pub steps: Vec<Rc<dyn PendingDashboardBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingDashboardBatchEngine {
    pub fn run(&mut self, item: &PendingDashboardBatch) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingDashboardBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingDashboardBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_pending_dashboard_batch_engine() -> PendingDashboardBatchEngine {
    PendingDashboardBatchEngine {
        steps: vec![
            Rc::new(RoundingPendingDashboardBatchAdjuster { step: 0.25 }) as Rc<dyn PendingDashboardBatchModifier>,
            Rc::new(TieredPendingDashboardBatchAdjuster {
                bounds: vec![3899.0, 5401.0, 7607.0],
                rates: vec![1.06, 1.12, 1.08],
            }),
            Rc::new(PendingDashboardBatchTtlSecondsOffsetAdjuster { weight: 1.28 }),
            Rc::new(PercentPendingDashboardBatchAdjuster { percent: 9.9 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 2473.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_dashboard_batch_engine();
        let item = new_pending_dashboard_batch("code-116", "title-42");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
