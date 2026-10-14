use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the budget burn profile calculation.
pub trait BudgetBurnProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnProfileFactor for TieredBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnProfileMinimumOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnProfileFactor for BudgetBurnProfileMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnProfile, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleBudgetBurnProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnProfileFactor for TitleBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &BudgetBurnProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnProfileAdjuster {
    pub step: f64,
}

impl BudgetBurnProfileFactor for RoundingBudgetBurnProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the budget burn profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnProfileEngine {
    pub steps: Vec<Rc<dyn BudgetBurnProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnProfileEngine {
    pub fn run(&mut self, item: &BudgetBurnProfile) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnProfileFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_budget_burn_profile_engine() -> BudgetBurnProfileEngine {
    BudgetBurnProfileEngine {
        steps: vec![
            Rc::new(TieredBudgetBurnProfileAdjuster {
                bounds: vec![525.0, 8008.0],
                rates: vec![0.88, 0.85],
            }) as Rc<dyn BudgetBurnProfileFactor>,
            Rc::new(BudgetBurnProfileMinimumOffsetAdjuster { weight: 0.71 }),
            Rc::new(TitleBudgetBurnProfileAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.93)]),
            }),
            Rc::new(RoundingBudgetBurnProfileAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 47041.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_profile_engine();
        let item = new_budget_burn_profile("code-71", "title-350");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
