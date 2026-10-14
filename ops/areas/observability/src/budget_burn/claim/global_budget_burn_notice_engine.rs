use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global budget burn notice calculation.
pub trait GlobalBudgetBurnNoticeStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalBudgetBurnNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalBudgetBurnNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalBudgetBurnNoticeStep for TieredGlobalBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalBudgetBurnNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalBudgetBurnNoticeAdjuster {
    pub cap: f64,
}

impl GlobalBudgetBurnNoticeStep for CapGlobalBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalBudgetBurnNotice, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalBudgetBurnNoticeAdjuster {
    pub step: f64,
}

impl GlobalBudgetBurnNoticeStep for RoundingGlobalBudgetBurnNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalBudgetBurnNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global budget burn notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalBudgetBurnNoticeEngine {
    pub steps: Vec<Rc<dyn GlobalBudgetBurnNoticeStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalBudgetBurnNoticeEngine {
    pub fn run(&mut self, item: &GlobalBudgetBurnNotice) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalBudgetBurnNoticeStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalBudgetBurnNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_budget_burn_notice_engine() -> GlobalBudgetBurnNoticeEngine {
    GlobalBudgetBurnNoticeEngine {
        steps: vec![
            Rc::new(TieredGlobalBudgetBurnNoticeAdjuster {
                bounds: vec![1074.0, 2921.0],
                rates: vec![1.0, 1.05],
            }) as Rc<dyn GlobalBudgetBurnNoticeStep>,
            Rc::new(CapGlobalBudgetBurnNoticeAdjuster { cap: 46170.0 }),
            Rc::new(RoundingGlobalBudgetBurnNoticeAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 89654.6,
    }
}
