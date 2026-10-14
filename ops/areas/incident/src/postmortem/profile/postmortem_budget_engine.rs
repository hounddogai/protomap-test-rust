use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the postmortem budget calculation.
pub trait PostmortemBudgetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PostmortemBudget, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPostmortemBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PostmortemBudgetModifier for FlaggedPostmortemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PostmortemBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudgetDurationMinutesOffsetAdjuster {
    pub weight: f64,
}

impl PostmortemBudgetModifier for PostmortemBudgetDurationMinutesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus duration minutes times {}", self.weight)
    }

    fn adjust(&self, item: &PostmortemBudget, value: f64) -> f64 {
        value + (item.duration_minutes as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PostmortemBudgetModifier for TieredPostmortemBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PostmortemBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the postmortem budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PostmortemBudgetEngine {
    pub steps: Vec<Rc<dyn PostmortemBudgetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PostmortemBudgetEngine {
    pub fn price(&mut self, item: &PostmortemBudget) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PostmortemBudgetModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PostmortemBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_postmortem_budget_engine(flags: &flags::Registry) -> PostmortemBudgetEngine {
    PostmortemBudgetEngine {
        steps: vec![
            Rc::new(FlaggedPostmortemBudgetAdjuster {
                flags: flags.clone(),
                flag: "postmortem-budget-pending".to_string(),
                percent: -5.9,
            }) as Rc<dyn PostmortemBudgetModifier>,
            Rc::new(PostmortemBudgetDurationMinutesOffsetAdjuster { weight: 1.58 }),
            Rc::new(TieredPostmortemBudgetAdjuster {
                bounds: vec![279.0, 5744.0, 7500.0],
                rates: vec![1.19, 0.89, 1.1],
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 57799.5,
    }
}
