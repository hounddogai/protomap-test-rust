use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the release budget calculation.
pub trait ReleaseBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReleaseBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseBudgetModifier for LabelReleaseBudgetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ReleaseBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseBudgetMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseBudgetModifier for ReleaseBudgetMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseBudget, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleaseBudgetModifier for TieredReleaseBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleaseBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the release budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseBudgetEngine {
    pub steps: Vec<Rc<dyn ReleaseBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseBudgetEngine {
    pub fn compute(&mut self, item: &ReleaseBudget) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_release_budget_engine() -> ReleaseBudgetEngine {
    ReleaseBudgetEngine {
        steps: vec![
            Rc::new(LabelReleaseBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.3), ("north".to_string(), 1.2)]),
            }) as Rc<dyn ReleaseBudgetModifier>,
            Rc::new(ReleaseBudgetMaximumOffsetAdjuster { weight: 1.51 }),
            Rc::new(TieredReleaseBudgetAdjuster {
                bounds: vec![4479.0, 4960.0, 7426.0],
                rates: vec![0.89, 0.93, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 1836.5,
    }
}
