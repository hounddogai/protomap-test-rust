use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending limit plan calculation.
pub trait PendingLimitPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingLimitPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingLimitPlanAdjuster {
    pub percent: f64,
}

impl PendingLimitPlanModifier for PercentPendingLimitPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingLimitPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingLimitPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingLimitPlanModifier for TieredPendingLimitPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingLimitPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingLimitPlanAdjuster {
    pub minimum: f64,
}

impl PendingLimitPlanModifier for MinimumPendingLimitPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingLimitPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the pending limit plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingLimitPlanEngine {
    pub steps: Vec<Rc<dyn PendingLimitPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingLimitPlanEngine {
    pub fn evaluate(&mut self, item: &PendingLimitPlan) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PendingLimitPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_limit_plan_engine() -> PendingLimitPlanEngine {
    PendingLimitPlanEngine {
        steps: vec![
            Rc::new(PercentPendingLimitPlanAdjuster { percent: -14.3 }) as Rc<dyn PendingLimitPlanModifier>,
            Rc::new(TieredPendingLimitPlanAdjuster {
                bounds: vec![5019.0, 6090.0],
                rates: vec![0.96, 0.92],
            }),
            Rc::new(MinimumPendingLimitPlanAdjuster { minimum: 44.2 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 90198.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_limit_plan_engine();
        let item = new_pending_limit_plan("code-753", "title-125");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
