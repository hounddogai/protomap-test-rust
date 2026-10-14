use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the action item baseline calculation.
pub trait ActionItemBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionActionItemBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemBaselineAdjuster for RegionActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &ActionItemBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemBaselineAdjuster {
    pub cap: f64,
}

impl ActionItemBaselineAdjuster for CapActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemBaselineAdjuster {
    pub minimum: f64,
}

impl ActionItemBaselineAdjuster for MinimumActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemBaselineAdjuster {
    pub step: f64,
}

impl ActionItemBaselineAdjuster for RoundingActionItemBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the action item baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemBaselineEngine {
    pub steps: Vec<Rc<dyn ActionItemBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemBaselineEngine {
    pub fn run(&mut self, item: &ActionItemBaseline) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_baseline_engine() -> ActionItemBaselineEngine {
    ActionItemBaselineEngine {
        steps: vec![
            Rc::new(RegionActionItemBaselineAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.04)]),
            }) as Rc<dyn ActionItemBaselineAdjuster>,
            Rc::new(CapActionItemBaselineAdjuster { cap: 14191.0 }),
            Rc::new(MinimumActionItemBaselineAdjuster { minimum: 15.2 }),
            Rc::new(RoundingActionItemBaselineAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 56381.7,
    }
}
