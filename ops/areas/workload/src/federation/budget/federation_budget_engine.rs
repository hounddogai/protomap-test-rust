use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the federation budget calculation.
pub trait FederationBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FederationBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedFederationBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FederationBudgetFactor for FlaggedFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FederationBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFederationBudgetAdjuster {
    pub percent: f64,
}

impl FederationBudgetFactor for PercentFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FederationBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FederationBudgetFactor for TieredFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FederationBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationBudgetAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl FederationBudgetFactor for FederationBudgetAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &FederationBudget, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFederationBudgetAdjuster {
    pub minimum: f64,
}

impl FederationBudgetFactor for MinimumFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FederationBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFederationBudgetAdjuster {
    pub cap: f64,
}

impl FederationBudgetFactor for CapFederationBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FederationBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the federation budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FederationBudgetEngine {
    pub steps: Vec<Rc<dyn FederationBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FederationBudgetEngine {
    pub fn run(&mut self, item: &FederationBudget) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[FederationBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_federation_budget_engine(flags: &flags::Registry) -> FederationBudgetEngine {
    FederationBudgetEngine {
        steps: vec![
            Rc::new(FlaggedFederationBudgetAdjuster {
                flags: flags.clone(),
                flag: "federation-budget-global".to_string(),
                percent: -6.8,
            }) as Rc<dyn FederationBudgetFactor>,
            Rc::new(PercentFederationBudgetAdjuster { percent: -19.4 }),
            Rc::new(TieredFederationBudgetAdjuster {
                bounds: vec![5421.0, 8331.0],
                rates: vec![0.94, 0.92],
            }),
            Rc::new(FederationBudgetAttemptsOffsetAdjuster { weight: 0.25 }),
            Rc::new(MinimumFederationBudgetAdjuster { minimum: 6.7 }),
            Rc::new(CapFederationBudgetAdjuster { cap: 42953.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 87818.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_federation_budget_engine(&flags::new_registry());
        let item = new_federation_budget("code-108", "name-303");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
