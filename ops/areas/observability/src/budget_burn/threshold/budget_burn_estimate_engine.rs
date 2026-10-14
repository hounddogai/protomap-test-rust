use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn estimate calculation.
pub trait BudgetBurnEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnEstimateAdjuster for FlaggedBudgetBurnEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnEstimateAdjuster for TieredBudgetBurnEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnEstimateAdjuster {
    pub minimum: f64,
}

impl BudgetBurnEstimateAdjuster for MinimumBudgetBurnEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnEstimatePortOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnEstimateAdjuster for BudgetBurnEstimatePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnEstimate, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnEstimateAdjuster {
    pub step: f64,
}

impl BudgetBurnEstimateAdjuster for RoundingBudgetBurnEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the budget burn estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnEstimateEngine {
    pub steps: Vec<Rc<dyn BudgetBurnEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnEstimateEngine {
    pub fn evaluate(&mut self, item: &BudgetBurnEstimate) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_budget_burn_estimate_engine(flags: &flags::Registry) -> BudgetBurnEstimateEngine {
    BudgetBurnEstimateEngine {
        steps: vec![
            Rc::new(FlaggedBudgetBurnEstimateAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-estimate-strict".to_string(),
                percent: -6.6,
            }) as Rc<dyn BudgetBurnEstimateAdjuster>,
            Rc::new(TieredBudgetBurnEstimateAdjuster {
                bounds: vec![926.0, 949.0, 2188.0, 3345.0],
                rates: vec![0.84, 0.96, 0.99, 1.2],
            }),
            Rc::new(MinimumBudgetBurnEstimateAdjuster { minimum: 17.8 }),
            Rc::new(BudgetBurnEstimatePortOffsetAdjuster { weight: 1.53 }),
            Rc::new(RoundingBudgetBurnEstimateAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 18451.2,
    }
}
