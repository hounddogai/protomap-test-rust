use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the grant budget calculation.
pub trait GrantBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantBudgetAdjuster {
    pub percent: f64,
}

impl GrantBudgetModifier for PercentGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionGrantBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantBudgetModifier for RegionGrantBudgetAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &GrantBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantBudgetAdjuster {
    pub step: f64,
}

impl GrantBudgetModifier for RoundingGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGrantBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GrantBudgetModifier for TieredGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GrantBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGrantBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GrantBudgetModifier for FlaggedGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GrantBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantBudgetAdjuster {
    pub minimum: f64,
}

impl GrantBudgetModifier for MinimumGrantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the grant budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantBudgetEngine {
    pub steps: Vec<Rc<dyn GrantBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantBudgetEngine {
    pub fn compute(&mut self, item: &GrantBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GrantBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_grant_budget_engine(flags: &flags::Registry) -> GrantBudgetEngine {
    GrantBudgetEngine {
        steps: vec![
            Rc::new(PercentGrantBudgetAdjuster { percent: -10.2 }) as Rc<dyn GrantBudgetModifier>,
            Rc::new(RegionGrantBudgetAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.14)]),
            }),
            Rc::new(RoundingGrantBudgetAdjuster { step: 0.1 }),
            Rc::new(TieredGrantBudgetAdjuster {
                bounds: vec![1257.0, 2644.0, 3191.0],
                rates: vec![1.03, 0.86, 0.97],
            }),
            Rc::new(FlaggedGrantBudgetAdjuster {
                flags: flags.clone(),
                flag: "grant-budget-adaptive".to_string(),
                percent: -8.3,
            }),
            Rc::new(MinimumGrantBudgetAdjuster { minimum: 18.2 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 79812.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_budget_engine(&flags::new_registry());
        let item = new_grant_budget("code-610", "name-359");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
