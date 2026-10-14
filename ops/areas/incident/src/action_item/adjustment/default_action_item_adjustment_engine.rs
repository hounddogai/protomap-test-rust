use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default action item adjustment calculation.
pub trait DefaultActionItemAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultActionItemAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultActionItemAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultActionItemAdjustmentFactor for FlaggedDefaultActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultActionItemAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultActionItemAdjustmentAdjuster {
    pub minimum: f64,
}

impl DefaultActionItemAdjustmentFactor for MinimumDefaultActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultActionItemAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultActionItemAdjustmentAdjuster {
    pub percent: f64,
}

impl DefaultActionItemAdjustmentFactor for PercentDefaultActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultActionItemAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default action item adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultActionItemAdjustmentEngineExtra {
    pub steps: Vec<Rc<dyn DefaultActionItemAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultActionItemAdjustmentEngineExtra {
    pub fn price(&mut self, item: &DefaultActionItemAdjustment) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DefaultActionItemAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_default_action_item_adjustment_engine(
    flags: &flags::Registry,
) -> DefaultActionItemAdjustmentEngineExtra {
    DefaultActionItemAdjustmentEngineExtra {
        steps: vec![
            Rc::new(FlaggedDefaultActionItemAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "default-action-item-adjustment-pending".to_string(),
                percent: -4.6,
            }) as Rc<dyn DefaultActionItemAdjustmentFactor>,
            Rc::new(MinimumDefaultActionItemAdjustmentAdjuster { minimum: 43.2 }),
            Rc::new(PercentDefaultActionItemAdjustmentAdjuster { percent: -13.4 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 63071.2,
    }
}
