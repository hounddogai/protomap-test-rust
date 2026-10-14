use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient rule draft calculation.
pub trait LenientRuleDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRuleDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientRuleDraftAdjuster {
    pub step: f64,
}

impl LenientRuleDraftStep for RoundingLenientRuleDraftAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientRuleDraft, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRuleDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRuleDraftStep for TieredLenientRuleDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRuleDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientRuleDraftAdjuster {
    pub percent: f64,
}

impl LenientRuleDraftStep for PercentLenientRuleDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientRuleDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitLenientRuleDraftAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientRuleDraftStep for UnitLenientRuleDraftAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &LenientRuleDraft, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the lenient rule draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRuleDraftEngine {
    pub steps: Vec<Rc<dyn LenientRuleDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRuleDraftEngine {
    pub fn price(&mut self, item: &LenientRuleDraft) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientRuleDraftStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientRuleDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_rule_draft_engine() -> LenientRuleDraftEngine {
    LenientRuleDraftEngine {
        steps: vec![
            Rc::new(RoundingLenientRuleDraftAdjuster { step: 1.0 }) as Rc<dyn LenientRuleDraftStep>,
            Rc::new(TieredLenientRuleDraftAdjuster {
                bounds: vec![446.0, 4079.0, 8803.0],
                rates: vec![1.03, 1.12, 0.98],
            }),
            Rc::new(PercentLenientRuleDraftAdjuster { percent: -0.3 }),
            Rc::new(UnitLenientRuleDraftAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.93), ("west".to_string(), 1.18)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 9882.1,
    }
}
