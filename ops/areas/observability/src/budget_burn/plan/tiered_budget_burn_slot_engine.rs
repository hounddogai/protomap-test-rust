use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered budget burn slot calculation.
pub trait TieredBudgetBurnSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredBudgetBurnSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredBudgetBurnSlotAdjuster {
    pub percent: f64,
}

impl TieredBudgetBurnSlotStep for PercentTieredBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredBudgetBurnSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredBudgetBurnSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredBudgetBurnSlotStep for TieredTieredBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredBudgetBurnSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnSlotCeilingOffsetAdjuster {
    pub weight: f64,
}

impl TieredBudgetBurnSlotStep for TieredBudgetBurnSlotCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &TieredBudgetBurnSlot, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the tiered budget burn slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredBudgetBurnSlotEngine {
    pub steps: Vec<Rc<dyn TieredBudgetBurnSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredBudgetBurnSlotEngine {
    pub fn run(&mut self, item: &TieredBudgetBurnSlot) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredBudgetBurnSlotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredBudgetBurnSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tiered_budget_burn_slot_engine() -> TieredBudgetBurnSlotEngine {
    TieredBudgetBurnSlotEngine {
        steps: vec![
            Rc::new(PercentTieredBudgetBurnSlotAdjuster { percent: -23.1 }) as Rc<dyn TieredBudgetBurnSlotStep>,
            Rc::new(TieredTieredBudgetBurnSlotAdjuster {
                bounds: vec![3028.0, 6110.0, 9651.0],
                rates: vec![1.11, 1.08, 0.91],
            }),
            Rc::new(TieredBudgetBurnSlotCeilingOffsetAdjuster { weight: 1.91 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 68661.9,
    }
}
