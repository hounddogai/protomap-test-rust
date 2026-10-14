use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal budget burn plan calculation.
pub trait SeasonalBudgetBurnPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalBudgetBurnPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalBudgetBurnPlanAdjuster {
    pub minimum: f64,
}

impl SeasonalBudgetBurnPlanModifier for MinimumSeasonalBudgetBurnPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalBudgetBurnPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalBudgetBurnPlanModifier for FlaggedSeasonalBudgetBurnPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalBudgetBurnPlanAdjuster {
    pub cap: f64,
}

impl SeasonalBudgetBurnPlanModifier for CapSeasonalBudgetBurnPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalBudgetBurnPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalBudgetBurnPlanModifier for TieredSeasonalBudgetBurnPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalBudgetBurnPlanAdjuster {
    pub step: f64,
}

impl SeasonalBudgetBurnPlanModifier for RoundingSeasonalBudgetBurnPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalBudgetBurnPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal budget burn plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnPlanEngine {
    pub steps: Vec<Rc<dyn SeasonalBudgetBurnPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalBudgetBurnPlanEngine {
    pub fn price(&mut self, item: &SeasonalBudgetBurnPlan) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalBudgetBurnPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_budget_burn_plan_engine(flags: &flags::Registry) -> SeasonalBudgetBurnPlanEngine {
    SeasonalBudgetBurnPlanEngine {
        steps: vec![
            Rc::new(MinimumSeasonalBudgetBurnPlanAdjuster { minimum: 10.2 }) as Rc<dyn SeasonalBudgetBurnPlanModifier>,
            Rc::new(FlaggedSeasonalBudgetBurnPlanAdjuster {
                flags: flags.clone(),
                flag: "seasonal-budget-burn-plan-tiered".to_string(),
                percent: 6.2,
            }),
            Rc::new(CapSeasonalBudgetBurnPlanAdjuster { cap: 46481.0 }),
            Rc::new(TieredSeasonalBudgetBurnPlanAdjuster {
                bounds: vec![2982.0, 4190.0, 7648.0, 8678.0],
                rates: vec![1.03, 1.17, 0.85, 0.82],
            }),
            Rc::new(RoundingSeasonalBudgetBurnPlanAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 53877.2,
    }
}
