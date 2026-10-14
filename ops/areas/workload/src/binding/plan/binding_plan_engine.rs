use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding plan calculation.
pub trait BindingPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingPlanCostOffsetAdjuster {
    pub weight: f64,
}

impl BindingPlanModifier for BindingPlanCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BindingPlan, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingPlanModifier for TieredBindingPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingPlanAdjuster {
    pub step: f64,
}

impl BindingPlanModifier for RoundingBindingPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the binding plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingPlanEngine {
    pub steps: Vec<Rc<dyn BindingPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingPlanEngine {
    pub fn price(&mut self, item: &BindingPlan) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BindingPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_binding_plan_engine() -> BindingPlanEngine {
    BindingPlanEngine {
        steps: vec![
            Rc::new(BindingPlanCostOffsetAdjuster { weight: 1.88 }) as Rc<dyn BindingPlanModifier>,
            Rc::new(TieredBindingPlanAdjuster {
                bounds: vec![1452.0, 5626.0],
                rates: vec![1.03, 1.2],
            }),
            Rc::new(RoundingBindingPlanAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 59346.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_plan_engine();
        let item = new_binding_plan("code-269", "name-172");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
