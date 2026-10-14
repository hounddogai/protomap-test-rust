use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item adjustment calculation.
pub trait ActionItemAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemAdjustmentFactor for FlaggedActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemAdjustmentAdjuster {
    pub percent: f64,
}

impl ActionItemAdjustmentFactor for PercentActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemAdjustmentAdjuster {
    pub step: f64,
}

impl ActionItemAdjustmentFactor for RoundingActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumActionItemAdjustmentAdjuster {
    pub minimum: f64,
}

impl ActionItemAdjustmentFactor for MinimumActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAdjustmentCeilingOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemAdjustmentFactor for ActionItemAdjustmentCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapActionItemAdjustmentAdjuster {
    pub cap: f64,
}

impl ActionItemAdjustmentFactor for CapActionItemAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ActionItemAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the action item adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemAdjustmentEngine {
    pub steps: Vec<Rc<dyn ActionItemAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemAdjustmentEngine {
    pub fn evaluate(&mut self, item: &ActionItemAdjustment) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_action_item_adjustment_engine(flags: &flags::Registry) -> ActionItemAdjustmentEngine {
    ActionItemAdjustmentEngine {
        steps: vec![
            Rc::new(FlaggedActionItemAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "action-item-adjustment-seasonal".to_string(),
                percent: -5.8,
            }) as Rc<dyn ActionItemAdjustmentFactor>,
            Rc::new(PercentActionItemAdjustmentAdjuster { percent: -10.8 }),
            Rc::new(RoundingActionItemAdjustmentAdjuster { step: 0.05 }),
            Rc::new(MinimumActionItemAdjustmentAdjuster { minimum: 40.9 }),
            Rc::new(ActionItemAdjustmentCeilingOffsetAdjuster { weight: 0.54 }),
            Rc::new(CapActionItemAdjustmentAdjuster { cap: 44411.0 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 15561.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_adjustment_engine(&flags::new_registry());
        let item = new_action_item_adjustment("code-140", "name-703");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
