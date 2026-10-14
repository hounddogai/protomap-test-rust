use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the budget burn threshold calculation.
pub trait BudgetBurnThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnThresholdAdjuster {
    pub cap: f64,
}

impl BudgetBurnThresholdAdjuster for CapBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnThresholdAdjuster {
    pub percent: f64,
}

impl BudgetBurnThresholdAdjuster for PercentBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleBudgetBurnThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnThresholdAdjuster for LocaleBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &BudgetBurnThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnThresholdAdjuster {
    pub minimum: f64,
}

impl BudgetBurnThresholdAdjuster for MinimumBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnThresholdAdjuster for TieredBudgetBurnThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the budget burn threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnThresholdEngine {
    pub steps: Vec<Rc<dyn BudgetBurnThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnThresholdEngine {
    pub fn run(&mut self, item: &BudgetBurnThreshold) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_budget_burn_threshold_engine() -> BudgetBurnThresholdEngine {
    BudgetBurnThresholdEngine {
        steps: vec![
            Rc::new(CapBudgetBurnThresholdAdjuster { cap: 26970.0 }) as Rc<dyn BudgetBurnThresholdAdjuster>,
            Rc::new(PercentBudgetBurnThresholdAdjuster { percent: -4.8 }),
            Rc::new(LocaleBudgetBurnThresholdAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.92),
                    ("online".to_string(), 0.78),
                    ("retail".to_string(), 0.74),
                ]),
            }),
            Rc::new(MinimumBudgetBurnThresholdAdjuster { minimum: 18.9 }),
            Rc::new(TieredBudgetBurnThresholdAdjuster {
                bounds: vec![1419.0, 4272.0],
                rates: vec![1.01, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 6177.2,
    }
}
