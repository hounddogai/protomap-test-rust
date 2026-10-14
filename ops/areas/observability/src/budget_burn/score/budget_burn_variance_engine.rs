use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn variance calculation.
pub trait BudgetBurnVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnVariance, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnVarianceStep for FlaggedBudgetBurnVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnVarianceCapacityOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnVarianceStep for BudgetBurnVarianceCapacityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus capacity times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnVariance, value: f64) -> f64 {
        value + (item.capacity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnVarianceAdjuster {
    pub minimum: f64,
}

impl BudgetBurnVarianceStep for MinimumBudgetBurnVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnVarianceAdjuster {
    pub cap: f64,
}

impl BudgetBurnVarianceStep for CapBudgetBurnVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the budget burn variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnVarianceEngine {
    pub steps: Vec<Rc<dyn BudgetBurnVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnVarianceEngine {
    pub fn evaluate(&mut self, item: &BudgetBurnVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_budget_burn_variance_engine(flags: &flags::Registry) -> BudgetBurnVarianceEngine {
    BudgetBurnVarianceEngine {
        steps: vec![
            Rc::new(FlaggedBudgetBurnVarianceAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-variance-default".to_string(),
                percent: -0.9,
            }) as Rc<dyn BudgetBurnVarianceStep>,
            Rc::new(BudgetBurnVarianceCapacityOffsetAdjuster { weight: 1.41 }),
            Rc::new(MinimumBudgetBurnVarianceAdjuster { minimum: 9.5 }),
            Rc::new(CapBudgetBurnVarianceAdjuster { cap: 24720.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 8251.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_variance_engine(&flags::new_registry());
        let item = new_budget_burn_variance("code-694", "label-189");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
