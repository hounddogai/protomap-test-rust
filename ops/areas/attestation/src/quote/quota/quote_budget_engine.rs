use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the quote budget calculation.
pub trait QuoteBudgetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &QuoteBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingQuoteBudgetAdjuster {
    pub step: f64,
}

impl QuoteBudgetFactor for RoundingQuoteBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &QuoteBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedQuoteBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl QuoteBudgetFactor for FlaggedQuoteBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &QuoteBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapQuoteBudgetAdjuster {
    pub cap: f64,
}

impl QuoteBudgetFactor for CapQuoteBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &QuoteBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl QuoteBudgetFactor for TieredQuoteBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &QuoteBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumQuoteBudgetAdjuster {
    pub minimum: f64,
}

impl QuoteBudgetFactor for MinimumQuoteBudgetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &QuoteBudget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the quote budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct QuoteBudgetEngine {
    pub steps: Vec<Rc<dyn QuoteBudgetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl QuoteBudgetEngine {
    pub fn compute(&mut self, item: &QuoteBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[QuoteBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_quote_budget_engine(flags: &flags::Registry) -> QuoteBudgetEngine {
    QuoteBudgetEngine {
        steps: vec![
            Rc::new(RoundingQuoteBudgetAdjuster { step: 0.5 }) as Rc<dyn QuoteBudgetFactor>,
            Rc::new(FlaggedQuoteBudgetAdjuster {
                flags: flags.clone(),
                flag: "quote-budget-pending".to_string(),
                percent: -3.7,
            }),
            Rc::new(CapQuoteBudgetAdjuster { cap: 21216.0 }),
            Rc::new(TieredQuoteBudgetAdjuster {
                bounds: vec![1241.0, 2442.0, 4970.0, 6971.0],
                rates: vec![0.99, 1.12, 1.1, 1.03],
            }),
            Rc::new(MinimumQuoteBudgetAdjuster { minimum: 47.8 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 27207.6,
    }
}
