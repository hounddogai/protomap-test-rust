use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional action item budget calculation.
pub trait RegionalActionItemBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalActionItemBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalActionItemBudgetAdjuster {
    pub percent: f64,
}

impl RegionalActionItemBudgetFactor for PercentRegionalActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalActionItemBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalActionItemBudgetAdjuster {
    pub step: f64,
}

impl RegionalActionItemBudgetFactor for RoundingRegionalActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalActionItemBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalActionItemBudgetAdjuster {
    pub minimum: f64,
}

impl RegionalActionItemBudgetFactor for MinimumRegionalActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalActionItemBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalActionItemBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalActionItemBudgetFactor for TieredRegionalActionItemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalActionItemBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemBudgetFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl RegionalActionItemBudgetFactor for RegionalActionItemBudgetFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalActionItemBudget, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

/// Runs the regional action item budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalActionItemBudgetEngine {
    pub steps: Vec<Rc<dyn RegionalActionItemBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalActionItemBudgetEngine {
    pub fn evaluate(&mut self, item: &RegionalActionItemBudget) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalActionItemBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalActionItemBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_action_item_budget_engine() -> RegionalActionItemBudgetEngine {
    RegionalActionItemBudgetEngine {
        steps: vec![
            Rc::new(PercentRegionalActionItemBudgetAdjuster { percent: -8.0 })
                as Rc<dyn RegionalActionItemBudgetFactor>,
            Rc::new(RoundingRegionalActionItemBudgetAdjuster { step: 0.05 }),
            Rc::new(MinimumRegionalActionItemBudgetAdjuster { minimum: 45.5 }),
            Rc::new(TieredRegionalActionItemBudgetAdjuster {
                bounds: vec![2975.0, 6863.0, 7329.0, 8831.0],
                rates: vec![0.88, 1.14, 1.08, 1.1],
            }),
            Rc::new(RegionalActionItemBudgetFloorValueOffsetAdjuster { weight: 1.19 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 90142.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_regional_action_item_budget_engine();
        let item = new_regional_action_item_budget("code-388", "name-910");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
