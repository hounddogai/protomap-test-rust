use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn slot calculation.
pub trait BudgetBurnSlotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnSlotAdjuster {
    pub step: f64,
}

impl BudgetBurnSlotFactor for RoundingBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryBudgetBurnSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnSlotFactor for CountryBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &BudgetBurnSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnSlotFactor for FlaggedBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnSlotFactor for TieredBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBudgetBurnSlotAdjuster {
    pub minimum: f64,
}

impl BudgetBurnSlotFactor for MinimumBudgetBurnSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BudgetBurnSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the budget burn slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnSlotEngine {
    pub steps: Vec<Rc<dyn BudgetBurnSlotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnSlotEngine {
    pub fn price(&mut self, item: &BudgetBurnSlot) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_slot_engine(flags: &flags::Registry) -> BudgetBurnSlotEngine {
    BudgetBurnSlotEngine {
        steps: vec![
            Rc::new(RoundingBudgetBurnSlotAdjuster { step: 0.25 }) as Rc<dyn BudgetBurnSlotFactor>,
            Rc::new(CountryBudgetBurnSlotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.11)]),
            }),
            Rc::new(FlaggedBudgetBurnSlotAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-slot-seasonal".to_string(),
                percent: -2.5,
            }),
            Rc::new(TieredBudgetBurnSlotAdjuster {
                bounds: vec![5828.0, 7689.0, 9995.0],
                rates: vec![0.97, 0.83, 0.84],
            }),
            Rc::new(MinimumBudgetBurnSlotAdjuster { minimum: 32.2 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 4167.2,
    }
}
