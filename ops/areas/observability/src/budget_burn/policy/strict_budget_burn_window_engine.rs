use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict budget burn window calculation.
pub trait StrictBudgetBurnWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictBudgetBurnWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictBudgetBurnWindowAdjuster {
    pub minimum: f64,
}

impl StrictBudgetBurnWindowFactor for MinimumStrictBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictBudgetBurnWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictBudgetBurnWindowAdjuster {
    pub step: f64,
}

impl StrictBudgetBurnWindowFactor for RoundingStrictBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictBudgetBurnWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictBudgetBurnWindowAdjuster {
    pub cap: f64,
}

impl StrictBudgetBurnWindowFactor for CapStrictBudgetBurnWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictBudgetBurnWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the strict budget burn window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictBudgetBurnWindowEngine {
    pub steps: Vec<Rc<dyn StrictBudgetBurnWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictBudgetBurnWindowEngine {
    pub fn run(&mut self, item: &StrictBudgetBurnWindow) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictBudgetBurnWindowFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictBudgetBurnWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_budget_burn_window_engine() -> StrictBudgetBurnWindowEngine {
    StrictBudgetBurnWindowEngine {
        steps: vec![
            Rc::new(MinimumStrictBudgetBurnWindowAdjuster { minimum: 49.8 }) as Rc<dyn StrictBudgetBurnWindowFactor>,
            Rc::new(RoundingStrictBudgetBurnWindowAdjuster { step: 1.0 }),
            Rc::new(CapStrictBudgetBurnWindowAdjuster { cap: 20603.0 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 50953.2,
    }
}
