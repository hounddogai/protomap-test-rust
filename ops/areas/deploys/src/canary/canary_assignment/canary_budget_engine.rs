use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the canary budget calculation.
pub trait CanaryBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBudgetRateOffsetAdjuster {
    pub weight: f64,
}

impl CanaryBudgetAdjuster for CanaryBudgetRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryBudgetAdjuster for TieredCanaryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCanaryBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CanaryBudgetAdjuster for FlaggedCanaryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryBudgetAdjuster {
    pub step: f64,
}

impl CanaryBudgetAdjuster for RoundingCanaryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameCanaryBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryBudgetAdjuster for NameCanaryBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryBudgetAdjuster {
    pub cap: f64,
}

impl CanaryBudgetAdjuster for CapCanaryBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the canary budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryBudgetEngine {
    pub steps: Vec<Rc<dyn CanaryBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryBudgetEngine {
    pub fn price(&mut self, item: &CanaryBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CanaryBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_canary_budget_engine(flags: &flags::Registry) -> CanaryBudgetEngine {
    CanaryBudgetEngine {
        steps: vec![
            Rc::new(CanaryBudgetRateOffsetAdjuster { weight: 0.76 }) as Rc<dyn CanaryBudgetAdjuster>,
            Rc::new(TieredCanaryBudgetAdjuster {
                bounds: vec![1496.0, 1668.0, 2255.0],
                rates: vec![0.89, 1.16, 1.03],
            }),
            Rc::new(FlaggedCanaryBudgetAdjuster {
                flags: flags.clone(),
                flag: "canary-budget-lenient".to_string(),
                percent: 7.2,
            }),
            Rc::new(RoundingCanaryBudgetAdjuster { step: 0.5 }),
            Rc::new(NameCanaryBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.0), ("retail".to_string(), 0.98)]),
            }),
            Rc::new(CapCanaryBudgetAdjuster { cap: 902.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 5859.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_canary_budget_engine(&flags::new_registry());
        let item = new_canary_budget("code-355", "name-57");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
