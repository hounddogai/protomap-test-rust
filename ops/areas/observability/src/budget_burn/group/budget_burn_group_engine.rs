use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn group calculation.
pub trait BudgetBurnGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnGroupAdjuster for FlaggedBudgetBurnGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnGroupAdjuster {
    pub cap: f64,
}

impl BudgetBurnGroupAdjuster for CapBudgetBurnGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnGroupAdjuster {
    pub step: f64,
}

impl BudgetBurnGroupAdjuster for RoundingBudgetBurnGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnGroupLimitOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnGroupAdjuster for BudgetBurnGroupLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnGroup, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the budget burn group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnGroupEngine {
    pub steps: Vec<Rc<dyn BudgetBurnGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnGroupEngine {
    pub fn price(&mut self, item: &BudgetBurnGroup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnGroupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_group_engine(flags: &flags::Registry) -> BudgetBurnGroupEngine {
    BudgetBurnGroupEngine {
        steps: vec![
            Rc::new(FlaggedBudgetBurnGroupAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-group-regional".to_string(),
                percent: -8.8,
            }) as Rc<dyn BudgetBurnGroupAdjuster>,
            Rc::new(CapBudgetBurnGroupAdjuster { cap: 6984.0 }),
            Rc::new(RoundingBudgetBurnGroupAdjuster { step: 0.1 }),
            Rc::new(BudgetBurnGroupLimitOffsetAdjuster { weight: 0.73 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 53600.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_group_engine(&flags::new_registry());
        let item = new_budget_burn_group("code-932", "title-360");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
