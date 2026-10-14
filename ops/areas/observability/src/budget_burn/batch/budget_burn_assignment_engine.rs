use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the budget burn assignment calculation.
pub trait BudgetBurnAssignmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnAssignmentAdjuster {
    pub cap: f64,
}

impl BudgetBurnAssignmentAdjuster for CapBudgetBurnAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnAssignmentAdjuster for TieredBudgetBurnAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnAssignmentCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnAssignmentAdjuster for BudgetBurnAssignmentCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnAssignment, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

/// Runs the budget burn assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnAssignmentEngine {
    pub steps: Vec<Rc<dyn BudgetBurnAssignmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnAssignmentEngine {
    pub fn price(&mut self, item: &BudgetBurnAssignment) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnAssignmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_assignment_engine() -> BudgetBurnAssignmentEngine {
    BudgetBurnAssignmentEngine {
        steps: vec![
            Rc::new(CapBudgetBurnAssignmentAdjuster { cap: 42509.0 }) as Rc<dyn BudgetBurnAssignmentAdjuster>,
            Rc::new(TieredBudgetBurnAssignmentAdjuster {
                bounds: vec![4397.0, 7488.0],
                rates: vec![0.95, 0.95],
            }),
            Rc::new(BudgetBurnAssignmentCreatedAtOffsetAdjuster { weight: 0.44 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 11688.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_assignment_engine();
        let item = new_budget_burn_assignment("code-348", "name-800");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
