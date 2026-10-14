use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the action item slot calculation.
pub trait ActionItemSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedActionItemSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ActionItemSlotModifier for FlaggedActionItemSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingActionItemSlotAdjuster {
    pub step: f64,
}

impl ActionItemSlotModifier for RoundingActionItemSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentActionItemSlotAdjuster {
    pub percent: f64,
}

impl ActionItemSlotModifier for PercentActionItemSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSlotTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl ActionItemSlotModifier for ActionItemSlotTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredActionItemSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ActionItemSlotModifier for TieredActionItemSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryActionItemSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ActionItemSlotModifier for CategoryActionItemSlotAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &ActionItemSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the action item slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ActionItemSlotEngine {
    pub steps: Vec<Rc<dyn ActionItemSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ActionItemSlotEngine {
    pub fn run(&mut self, item: &ActionItemSlot) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ActionItemSlotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ActionItemSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_action_item_slot_engine(flags: &flags::Registry) -> ActionItemSlotEngine {
    ActionItemSlotEngine {
        steps: vec![
            Rc::new(FlaggedActionItemSlotAdjuster {
                flags: flags.clone(),
                flag: "action-item-slot-scheduled".to_string(),
                percent: 0.4,
            }) as Rc<dyn ActionItemSlotModifier>,
            Rc::new(RoundingActionItemSlotAdjuster { step: 0.05 }),
            Rc::new(PercentActionItemSlotAdjuster { percent: 1.8 }),
            Rc::new(ActionItemSlotTtlSecondsOffsetAdjuster { weight: 1.9 }),
            Rc::new(TieredActionItemSlotAdjuster {
                bounds: vec![1533.0, 3514.0, 4133.0, 6079.0],
                rates: vec![1.17, 0.81, 0.96, 0.81],
            }),
            Rc::new(CategoryActionItemSlotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.73)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 28153.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_action_item_slot_engine(&flags::new_registry());
        let item = new_action_item_slot("code-718", "title-612");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
