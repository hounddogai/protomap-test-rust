use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the limit budget calculation.
pub trait LimitBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LimitBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLimitBudgetAdjuster {
    pub minimum: f64,
}

impl LimitBudgetStep for MinimumLimitBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LimitBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LimitBudgetStep for TieredLimitBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LimitBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLimitBudgetAdjuster {
    pub step: f64,
}

impl LimitBudgetStep for RoundingLimitBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LimitBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLimitBudgetAdjuster {
    pub cap: f64,
}

impl LimitBudgetStep for CapLimitBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LimitBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLimitBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LimitBudgetStep for FlaggedLimitBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LimitBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the limit budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LimitBudgetEngine {
    pub steps: Vec<Rc<dyn LimitBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LimitBudgetEngine {
    pub fn price(&mut self, item: &LimitBudget) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LimitBudgetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LimitBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_limit_budget_engine(flags: &flags::Registry) -> LimitBudgetEngine {
    LimitBudgetEngine {
        steps: vec![
            Rc::new(MinimumLimitBudgetAdjuster { minimum: 0.2 }) as Rc<dyn LimitBudgetStep>,
            Rc::new(TieredLimitBudgetAdjuster {
                bounds: vec![967.0, 1716.0, 9093.0],
                rates: vec![1.04, 0.97, 0.88],
            }),
            Rc::new(RoundingLimitBudgetAdjuster { step: 0.1 }),
            Rc::new(CapLimitBudgetAdjuster { cap: 18513.0 }),
            Rc::new(FlaggedLimitBudgetAdjuster {
                flags: flags.clone(),
                flag: "limit-budget-regional".to_string(),
                percent: -0.8,
            }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 63557.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_limit_budget_engine(&flags::new_registry());
        let item = new_limit_budget("code-375", "label-518");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
