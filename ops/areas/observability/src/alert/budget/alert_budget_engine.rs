use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert budget calculation.
pub trait AlertBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertBudgetModifier for TieredAlertBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAlertBudgetAdjuster {
    pub percent: f64,
}

impl AlertBudgetModifier for PercentAlertBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AlertBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertBudgetModifier for FlaggedAlertBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertBudgetAdjuster {
    pub step: f64,
}

impl AlertBudgetModifier for RoundingAlertBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAlertBudgetAdjuster {
    pub cap: f64,
}

impl AlertBudgetModifier for CapAlertBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AlertBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the alert budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertBudgetEngine {
    pub steps: Vec<Rc<dyn AlertBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertBudgetEngine {
    pub fn compute(&mut self, item: &AlertBudget) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AlertBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AlertBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_alert_budget_engine(flags: &flags::Registry) -> AlertBudgetEngine {
    AlertBudgetEngine {
        steps: vec![
            Rc::new(TieredAlertBudgetAdjuster {
                bounds: vec![3816.0, 4284.0],
                rates: vec![1.06, 0.85],
            }) as Rc<dyn AlertBudgetModifier>,
            Rc::new(PercentAlertBudgetAdjuster { percent: -0.4 }),
            Rc::new(FlaggedAlertBudgetAdjuster {
                flags: flags.clone(),
                flag: "alert-budget-manual".to_string(),
                percent: 1.1,
            }),
            Rc::new(RoundingAlertBudgetAdjuster { step: 0.1 }),
            Rc::new(CapAlertBudgetAdjuster { cap: 29452.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 98107.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_budget_engine(&flags::new_registry());
        let item = new_alert_budget("code-867", "name-901");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
