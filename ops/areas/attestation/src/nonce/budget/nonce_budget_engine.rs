use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the nonce budget calculation.
pub trait NonceBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceBudgetGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl NonceBudgetModifier for NonceBudgetGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &NonceBudget, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceBudgetAdjuster {
    pub percent: f64,
}

impl NonceBudgetModifier for PercentNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedNonceBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl NonceBudgetModifier for FlaggedNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &NonceBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceBudgetAdjuster {
    pub cap: f64,
}

impl NonceBudgetModifier for CapNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceBudgetAdjuster {
    pub step: f64,
}

impl NonceBudgetModifier for RoundingNonceBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the nonce budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceBudgetEngine {
    pub steps: Vec<Rc<dyn NonceBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceBudgetEngine {
    pub fn evaluate(&mut self, item: &NonceBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[NonceBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_nonce_budget_engine(flags: &flags::Registry) -> NonceBudgetEngine {
    NonceBudgetEngine {
        steps: vec![
            Rc::new(NonceBudgetGraceDaysOffsetAdjuster { weight: 0.13 }) as Rc<dyn NonceBudgetModifier>,
            Rc::new(PercentNonceBudgetAdjuster { percent: -11.2 }),
            Rc::new(FlaggedNonceBudgetAdjuster {
                flags: flags.clone(),
                flag: "nonce-budget-rolling".to_string(),
                percent: 4.6,
            }),
            Rc::new(CapNonceBudgetAdjuster { cap: 48689.0 }),
            Rc::new(RoundingNonceBudgetAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 15763.6,
    }
}
