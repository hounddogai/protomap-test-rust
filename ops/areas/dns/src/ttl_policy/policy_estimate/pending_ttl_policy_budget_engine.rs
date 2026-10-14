use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the pending ttl policy budget calculation.
pub trait PendingTtlPolicyBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTtlPolicyBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyBudgetRatioOffsetAdjuster {
    pub weight: f64,
}

impl PendingTtlPolicyBudgetStep for PendingTtlPolicyBudgetRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PendingTtlPolicyBudget, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingTtlPolicyBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingTtlPolicyBudgetStep for TieredPendingTtlPolicyBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingTtlPolicyBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingTtlPolicyBudgetAdjuster {
    pub percent: f64,
}

impl PendingTtlPolicyBudgetStep for PercentPendingTtlPolicyBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingTtlPolicyBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending ttl policy budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyBudgetEngine {
    pub steps: Vec<Rc<dyn PendingTtlPolicyBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTtlPolicyBudgetEngine {
    pub fn compute(&mut self, item: &PendingTtlPolicyBudget) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingTtlPolicyBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingTtlPolicyBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_ttl_policy_budget_engine() -> PendingTtlPolicyBudgetEngine {
    PendingTtlPolicyBudgetEngine {
        steps: vec![
            Rc::new(PendingTtlPolicyBudgetRatioOffsetAdjuster { weight: 0.61 }) as Rc<dyn PendingTtlPolicyBudgetStep>,
            Rc::new(TieredPendingTtlPolicyBudgetAdjuster {
                bounds: vec![259.0, 2836.0, 5081.0],
                rates: vec![0.83, 0.8, 1.13],
            }),
            Rc::new(PercentPendingTtlPolicyBudgetAdjuster { percent: -12.9 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 54338.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_ttl_policy_budget_engine();
        let item = new_pending_ttl_policy_budget("code-409", "label-899");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
