use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the binding budget calculation.
pub trait BindingBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingBudgetAdjuster {
    pub minimum: f64,
}

impl BindingBudgetModifier for MinimumBindingBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingBudgetUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl BindingBudgetModifier for BindingBudgetUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &BindingBudget, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingBudgetModifier for TieredBindingBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBindingBudgetAdjuster {
    pub cap: f64,
}

impl BindingBudgetModifier for CapBindingBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BindingBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingBudgetAdjuster {
    pub percent: f64,
}

impl BindingBudgetModifier for PercentBindingBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the binding budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingBudgetEngine {
    pub steps: Vec<Rc<dyn BindingBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingBudgetEngine {
    pub fn evaluate(&mut self, item: &BindingBudget) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_budget_engine() -> BindingBudgetEngine {
    BindingBudgetEngine {
        steps: vec![
            Rc::new(MinimumBindingBudgetAdjuster { minimum: 17.0 }) as Rc<dyn BindingBudgetModifier>,
            Rc::new(BindingBudgetUpdatedAtOffsetAdjuster { weight: 1.7 }),
            Rc::new(TieredBindingBudgetAdjuster {
                bounds: vec![7897.0, 8724.0],
                rates: vec![0.98, 0.92],
            }),
            Rc::new(CapBindingBudgetAdjuster { cap: 37874.0 }),
            Rc::new(PercentBindingBudgetAdjuster { percent: -16.9 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 9413.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_binding_budget_engine();
        let item = new_binding_budget("code-499", "title-185");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
