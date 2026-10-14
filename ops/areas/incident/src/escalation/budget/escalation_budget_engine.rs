use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the escalation budget calculation.
pub trait EscalationBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EscalationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudgetMinimumOffsetAdjuster {
    pub weight: f64,
}

impl EscalationBudgetAdjuster for EscalationBudgetMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &EscalationBudget, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEscalationBudgetAdjuster {
    pub step: f64,
}

impl EscalationBudgetAdjuster for RoundingEscalationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EscalationBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl EscalationBudgetAdjuster for TieredEscalationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &EscalationBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the escalation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EscalationBudgetEngine {
    pub steps: Vec<Rc<dyn EscalationBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EscalationBudgetEngine {
    pub fn price(&mut self, item: &EscalationBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EscalationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_escalation_budget_engine() -> EscalationBudgetEngine {
    EscalationBudgetEngine {
        steps: vec![
            Rc::new(EscalationBudgetMinimumOffsetAdjuster { weight: 1.2 }) as Rc<dyn EscalationBudgetAdjuster>,
            Rc::new(RoundingEscalationBudgetAdjuster { step: 0.1 }),
            Rc::new(TieredEscalationBudgetAdjuster {
                bounds: vec![3356.0, 8673.0],
                rates: vec![0.98, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 81747.3,
    }
}
