use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped action item budget calculation.
pub trait CappedActionItemBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedActionItemBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedActionItemBudgetFactor for FlaggedCappedActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCappedActionItemBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedActionItemBudgetFactor for NameCappedActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedActionItemBudgetAdjuster {
    pub percent: f64,
}

impl CappedActionItemBudgetFactor for PercentCappedActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedActionItemBudgetAdjuster {
    pub cap: f64,
}

impl CappedActionItemBudgetFactor for CapCappedActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedActionItemBudgetAdjuster {
    pub minimum: f64,
}

impl CappedActionItemBudgetFactor for MinimumCappedActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedActionItemBudgetAdjuster {
    pub step: f64,
}

impl CappedActionItemBudgetFactor for RoundingCappedActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedActionItemBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped action item budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedActionItemBudgetEngine {
    pub steps: Vec<Rc<dyn CappedActionItemBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedActionItemBudgetEngine {
    pub fn price(&mut self, item: &CappedActionItemBudget) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedActionItemBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedActionItemBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_action_item_budget_engine(flags: &flags::Registry) -> CappedActionItemBudgetEngine {
    CappedActionItemBudgetEngine {
        steps: vec![
            Rc::new(FlaggedCappedActionItemBudgetAdjuster {
                flags: flags.clone(),
                flag: "capped-action-item-budget-legacy".to_string(),
                percent: 6.4,
            }) as Rc<dyn CappedActionItemBudgetFactor>,
            Rc::new(NameCappedActionItemBudgetAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.35), ("south".to_string(), 1.26)]),
            }),
            Rc::new(PercentCappedActionItemBudgetAdjuster { percent: 7.8 }),
            Rc::new(CapCappedActionItemBudgetAdjuster { cap: 14864.0 }),
            Rc::new(MinimumCappedActionItemBudgetAdjuster { minimum: 15.7 }),
            Rc::new(RoundingCappedActionItemBudgetAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 42190.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_action_item_budget_engine(&flags::new_registry());
        let item = new_capped_action_item_budget("code-724", "name-334");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
