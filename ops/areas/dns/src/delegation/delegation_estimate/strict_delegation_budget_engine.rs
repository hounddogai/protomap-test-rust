use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict delegation budget calculation.
pub trait StrictDelegationBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictDelegationBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictDelegationBudgetModifier for FlaggedStrictDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationBudgetFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl StrictDelegationBudgetModifier for StrictDelegationBudgetFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictDelegationBudgetAdjuster {
    pub minimum: f64,
}

impl StrictDelegationBudgetModifier for MinimumStrictDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictDelegationBudgetAdjuster {
    pub percent: f64,
}

impl StrictDelegationBudgetModifier for PercentStrictDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictDelegationBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictDelegationBudgetModifier for TieredStrictDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictDelegationBudgetAdjuster {
    pub cap: f64,
}

impl StrictDelegationBudgetModifier for CapStrictDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictDelegationBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict delegation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictDelegationBudgetEngine {
    pub steps: Vec<Rc<dyn StrictDelegationBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictDelegationBudgetEngine {
    pub fn compute(&mut self, item: &StrictDelegationBudget) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[StrictDelegationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_strict_delegation_budget_engine(flags: &flags::Registry) -> StrictDelegationBudgetEngine {
    StrictDelegationBudgetEngine {
        steps: vec![
            Rc::new(FlaggedStrictDelegationBudgetAdjuster {
                flags: flags.clone(),
                flag: "strict-delegation-budget-pending".to_string(),
                percent: 8.1,
            }) as Rc<dyn StrictDelegationBudgetModifier>,
            Rc::new(StrictDelegationBudgetFloorValueOffsetAdjuster { weight: 0.82 }),
            Rc::new(MinimumStrictDelegationBudgetAdjuster { minimum: 27.0 }),
            Rc::new(PercentStrictDelegationBudgetAdjuster { percent: 2.0 }),
            Rc::new(TieredStrictDelegationBudgetAdjuster {
                bounds: vec![6882.0, 7282.0],
                rates: vec![1.18, 0.86],
            }),
            Rc::new(CapStrictDelegationBudgetAdjuster { cap: 2893.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 90906.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_delegation_budget_engine(&flags::new_registry());
        let item = new_strict_delegation_budget("code-871", "name-658");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
