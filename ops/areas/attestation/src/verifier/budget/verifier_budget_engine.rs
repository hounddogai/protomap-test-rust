use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the verifier budget calculation.
pub trait VerifierBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedVerifierBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl VerifierBudgetFactor for FlaggedVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameVerifierBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierBudgetFactor for NameVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBudgetRatioOffsetAdjuster {
    pub weight: f64,
}

impl VerifierBudgetFactor for VerifierBudgetRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierBudgetAdjuster {
    pub cap: f64,
}

impl VerifierBudgetFactor for CapVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierBudgetFactor for TieredVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierBudgetAdjuster {
    pub minimum: f64,
}

impl VerifierBudgetFactor for MinimumVerifierBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the verifier budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierBudgetEngine {
    pub steps: Vec<Rc<dyn VerifierBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierBudgetEngine {
    pub fn price(&mut self, item: &VerifierBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[VerifierBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_verifier_budget_engine(flags: &flags::Registry) -> VerifierBudgetEngine {
    VerifierBudgetEngine {
        steps: vec![
            Rc::new(FlaggedVerifierBudgetAdjuster {
                flags: flags.clone(),
                flag: "verifier-budget-global".to_string(),
                percent: -9.8,
            }) as Rc<dyn VerifierBudgetFactor>,
            Rc::new(NameVerifierBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.94)]),
            }),
            Rc::new(VerifierBudgetRatioOffsetAdjuster { weight: 1.92 }),
            Rc::new(CapVerifierBudgetAdjuster { cap: 23916.0 }),
            Rc::new(TieredVerifierBudgetAdjuster {
                bounds: vec![9043.0, 9554.0],
                rates: vec![1.0, 0.97],
            }),
            Rc::new(MinimumVerifierBudgetAdjuster { minimum: 10.9 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 70325.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_verifier_budget_engine(&flags::new_registry());
        let item = new_verifier_budget("code-745", "name-72");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
