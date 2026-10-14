use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the budget burn template calculation.
pub trait BudgetBurnTemplateModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTemplateCostOffsetAdjuster {
    pub weight: f64,
}

impl BudgetBurnTemplateModifier for BudgetBurnTemplateCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBudgetBurnTemplateAdjuster {
    pub step: f64,
}

impl BudgetBurnTemplateModifier for RoundingBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBudgetBurnTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BudgetBurnTemplateModifier for FlaggedBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnTemplateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BudgetBurnTemplateModifier for TieredBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBudgetBurnTemplateAdjuster {
    pub percent: f64,
}

impl BudgetBurnTemplateModifier for PercentBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeBudgetBurnTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BudgetBurnTemplateModifier for ScopeBudgetBurnTemplateAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &BudgetBurnTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the budget burn template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BudgetBurnTemplateEngine {
    pub steps: Vec<Rc<dyn BudgetBurnTemplateModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BudgetBurnTemplateEngine {
    pub fn price(&mut self, item: &BudgetBurnTemplate) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BudgetBurnTemplateModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BudgetBurnTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_budget_burn_template_engine(flags: &flags::Registry) -> BudgetBurnTemplateEngine {
    BudgetBurnTemplateEngine {
        steps: vec![
            Rc::new(BudgetBurnTemplateCostOffsetAdjuster { weight: 0.79 }) as Rc<dyn BudgetBurnTemplateModifier>,
            Rc::new(RoundingBudgetBurnTemplateAdjuster { step: 0.5 }),
            Rc::new(FlaggedBudgetBurnTemplateAdjuster {
                flags: flags.clone(),
                flag: "budget-burn-template-legacy".to_string(),
                percent: -6.1,
            }),
            Rc::new(TieredBudgetBurnTemplateAdjuster {
                bounds: vec![3311.0, 4536.0, 6139.0],
                rates: vec![0.93, 1.11, 0.86],
            }),
            Rc::new(PercentBudgetBurnTemplateAdjuster { percent: 7.1 }),
            Rc::new(ScopeBudgetBurnTemplateAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.73), ("online".to_string(), 0.86)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 38633.4,
    }
}
