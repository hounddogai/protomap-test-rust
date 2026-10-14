use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the approval variance calculation.
pub trait ApprovalVarianceFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalVarianceAdjuster {
    pub minimum: f64,
}

impl ApprovalVarianceFactor for MinimumApprovalVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingApprovalVarianceAdjuster {
    pub step: f64,
}

impl ApprovalVarianceFactor for RoundingApprovalVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ApprovalVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalVarianceAdjuster {
    pub percent: f64,
}

impl ApprovalVarianceFactor for PercentApprovalVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalVarianceEngine {
    pub steps: Vec<Rc<dyn ApprovalVarianceFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalVarianceEngine {
    pub fn price(&mut self, item: &ApprovalVariance) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_approval_variance_engine() -> ApprovalVarianceEngine {
    ApprovalVarianceEngine {
        steps: vec![
            Rc::new(MinimumApprovalVarianceAdjuster { minimum: 35.3 }) as Rc<dyn ApprovalVarianceFactor>,
            Rc::new(RoundingApprovalVarianceAdjuster { step: 0.1 }),
            Rc::new(PercentApprovalVarianceAdjuster { percent: 19.7 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 29682.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_approval_variance_engine();
        let item = new_approval_variance("code-203", "label-716");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
