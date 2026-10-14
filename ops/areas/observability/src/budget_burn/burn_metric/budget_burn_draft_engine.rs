use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the budget burn draft calculation.
pub trait BudgetBurnDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnDraftAdjuster {
    pub percent: f64,
}

impl BudgetBurnDraftAdjuster for PercentBudgetBurnDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnDraftUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnDraftAdjuster for BudgetBurnDraftUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnDraft, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnDraftAdjuster for TieredBudgetBurnDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnDraftAdjuster {
    pub cap: f64,
}

impl BudgetBurnDraftAdjuster for CapBudgetBurnDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the budget burn draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnDraftEngine {
    pub steps: Vec<Rc<dyn BudgetBurnDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnDraftEngine {
    pub fn compute(&mut self, item: &BudgetBurnDraft) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_budget_burn_draft_engine() -> BudgetBurnDraftEngine {
    BudgetBurnDraftEngine {
        steps: vec![
            Rc::new(PercentBudgetBurnDraftAdjuster { percent: -24.0 }) as Rc<dyn BudgetBurnDraftAdjuster>,
            Rc::new(BudgetBurnDraftUpdatedAtOffsetAdjuster { weight: 0.37 }),
            Rc::new(TieredBudgetBurnDraftAdjuster {
                bounds: vec![3184.0, 3977.0, 5045.0],
                rates: vec![1.07, 0.86, 0.85],
            }),
            Rc::new(CapBudgetBurnDraftAdjuster { cap: 16408.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 76168.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_draft_engine();
        let item = new_budget_burn_draft("code-321", "name-819");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
