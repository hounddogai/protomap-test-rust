use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the delegation budget calculation.
pub trait DelegationBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationBudgetAdjuster {
    pub minimum: f64,
}

impl DelegationBudgetFactor for MinimumDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationBudgetAdjuster {
    pub percent: f64,
}

impl DelegationBudgetFactor for PercentDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBudgetAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl DelegationBudgetFactor for DelegationBudgetAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationBudget, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDelegationBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DelegationBudgetFactor for FlaggedDelegationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DelegationBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationBudgetEngine {
    pub steps: Vec<Rc<dyn DelegationBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationBudgetEngine {
    pub fn compute(&mut self, item: &DelegationBudget) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DelegationBudgetFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DelegationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_delegation_budget_engine(flags: &flags::Registry) -> DelegationBudgetEngine {
    DelegationBudgetEngine {
        steps: vec![
            Rc::new(MinimumDelegationBudgetAdjuster { minimum: 47.1 }) as Rc<dyn DelegationBudgetFactor>,
            Rc::new(PercentDelegationBudgetAdjuster { percent: -8.5 }),
            Rc::new(DelegationBudgetAttemptsOffsetAdjuster { weight: 1.02 }),
            Rc::new(FlaggedDelegationBudgetAdjuster {
                flags: flags.clone(),
                flag: "delegation-budget-seasonal".to_string(),
                percent: 5.5,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 78233.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_budget_engine(&flags::new_registry());
        let item = new_delegation_budget("code-526", "name-383");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
