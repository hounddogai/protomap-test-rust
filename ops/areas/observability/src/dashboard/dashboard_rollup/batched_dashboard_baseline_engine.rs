use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched dashboard baseline calculation.
pub trait BatchedDashboardBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedDashboardBaselineAdjuster {
    pub step: f64,
}

impl BatchedDashboardBaselineFactor for RoundingBatchedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedDashboardBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedDashboardBaselineFactor for TieredBatchedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedDashboardBaselineAdjuster {
    pub minimum: f64,
}

impl BatchedDashboardBaselineFactor for MinimumBatchedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionBatchedDashboardBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedDashboardBaselineFactor for RegionBatchedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedDashboardBaselineAdjuster {
    pub cap: f64,
}

impl BatchedDashboardBaselineFactor for CapBatchedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedDashboardBaselineAdjuster {
    pub percent: f64,
}

impl BatchedDashboardBaselineFactor for PercentBatchedDashboardBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedDashboardBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the batched dashboard baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedDashboardBaselineEngine {
    pub steps: Vec<Rc<dyn BatchedDashboardBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedDashboardBaselineEngine {
    pub fn compute(&mut self, item: &BatchedDashboardBaseline) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedDashboardBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_batched_dashboard_baseline_engine() -> BatchedDashboardBaselineEngine {
    BatchedDashboardBaselineEngine {
        steps: vec![
            Rc::new(RoundingBatchedDashboardBaselineAdjuster { step: 0.5 }) as Rc<dyn BatchedDashboardBaselineFactor>,
            Rc::new(TieredBatchedDashboardBaselineAdjuster {
                bounds: vec![3963.0, 4923.0, 6196.0, 6325.0],
                rates: vec![0.84, 1.07, 0.96, 0.85],
            }),
            Rc::new(MinimumBatchedDashboardBaselineAdjuster { minimum: 27.7 }),
            Rc::new(RegionBatchedDashboardBaselineAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.3),
                    ("central".to_string(), 1.0),
                    ("online".to_string(), 1.21),
                ]),
            }),
            Rc::new(CapBatchedDashboardBaselineAdjuster { cap: 11968.0 }),
            Rc::new(PercentBatchedDashboardBaselineAdjuster { percent: 23.2 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 79502.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_dashboard_baseline_engine();
        let item = new_batched_dashboard_baseline("code-183", "label-245");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
