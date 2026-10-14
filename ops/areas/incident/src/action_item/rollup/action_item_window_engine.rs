use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item window calculation.
pub trait ActionItemWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemWindowAdjuster {
    pub step: f64,
}

impl ActionItemWindowFactor for RoundingActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemWindowExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemWindowFactor for ActionItemWindowExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemWindowFactor for FlaggedActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemWindowAdjuster {
    pub cap: f64,
}

impl ActionItemWindowFactor for CapActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemWindowFactor for TieredActionItemWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelActionItemWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemWindowFactor for LabelActionItemWindowAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ActionItemWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the action item window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemWindowEngine {
    pub steps: Vec<Rc<dyn ActionItemWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemWindowEngine {
    pub fn price(&mut self, item: &ActionItemWindow) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_action_item_window_engine(flags: &flags::Registry) -> ActionItemWindowEngine {
    ActionItemWindowEngine {
        steps: vec![
            Rc::new(RoundingActionItemWindowAdjuster { step: 0.25 }) as Rc<dyn ActionItemWindowFactor>,
            Rc::new(ActionItemWindowExpiresAtOffsetAdjuster { weight: 1.04 }),
            Rc::new(FlaggedActionItemWindowAdjuster {
                flags: flags.clone(),
                flag: "action-item-window-global".to_string(),
                percent: -3.0,
            }),
            Rc::new(CapActionItemWindowAdjuster { cap: 28481.0 }),
            Rc::new(TieredActionItemWindowAdjuster {
                bounds: vec![4546.0, 4828.0, 5555.0, 8591.0],
                rates: vec![0.91, 1.09, 1.09, 0.86],
            }),
            Rc::new(LabelActionItemWindowAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 1.28),
                    ("west".to_string(), 1.11),
                    ("south".to_string(), 1.12),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 27250.8,
    }
}
