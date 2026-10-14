use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the budget burn score calculation.
pub trait BudgetBurnScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnScoreAdjuster for FlaggedBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBudgetBurnScoreAdjuster {
    pub cap: f64,
}

impl BudgetBurnScoreAdjuster for CapBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnScoreAdjuster {
    pub minimum: f64,
}

impl BudgetBurnScoreAdjuster for MinimumBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnScoreMinimumOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnScoreAdjuster for BudgetBurnScoreMinimumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus minimum times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64 {
        value + item.minimum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnScoreAdjuster for TieredBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnScoreAdjuster {
    pub percent: f64,
}

impl BudgetBurnScoreAdjuster for PercentBudgetBurnScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the budget burn score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnScoreEngine {
    pub steps: Vec<Rc<dyn BudgetBurnScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnScoreEngine {
    pub fn run(&mut self, item: &BudgetBurnScore) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_budget_burn_score_engine(flags: &flags::Registry) -> BudgetBurnScoreEngine {
    BudgetBurnScoreEngine {
        steps: vec![
            Rc::new(FlaggedBudgetBurnScoreAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-score-default".to_string(),
                percent: -3.2,
            }) as Rc<dyn BudgetBurnScoreAdjuster>,
            Rc::new(CapBudgetBurnScoreAdjuster { cap: 14044.0 }),
            Rc::new(MinimumBudgetBurnScoreAdjuster { minimum: 49.7 }),
            Rc::new(BudgetBurnScoreMinimumOffsetAdjuster { weight: 0.92 }),
            Rc::new(TieredBudgetBurnScoreAdjuster {
                bounds: vec![170.0, 732.0, 2251.0],
                rates: vec![0.81, 1.05, 1.1],
            }),
            Rc::new(PercentBudgetBurnScoreAdjuster { percent: -1.8 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 37211.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_budget_burn_score_engine(&flags::new_registry());
        let item = new_budget_burn_score("code-890", "name-150");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
